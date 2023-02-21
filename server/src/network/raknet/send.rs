use std::io::Write;
use std::sync::atomic::Ordering;

use async_recursion::async_recursion;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use flate2::write::DeflateEncoder;
use flate2::Compression;

use crate::config::SERVER_CONFIG;
use crate::network::header::Header;
use crate::network::packets::login::CompressionAlgorithm;
use crate::network::packets::{ConnectedPacket, Packet, CONNECTED_PACKET_ID};
use crate::network::raknet::packets::{Ack, AckRecord};
use crate::network::raknet::Reliability;
use crate::network::raknet::{Frame, FrameBatch};
use crate::network::session::Session;
use common::ReadExtensions;
use common::VResult;
use common::{Deserialize, Serialize};

use super::SendPriority;

pub struct PacketConfig {
    pub reliability: Reliability,
    pub priority: SendPriority,
}

pub const DEFAULT_SEND_CONFIG: PacketConfig = PacketConfig {
    reliability: Reliability::ReliableOrdered,
    priority: SendPriority::Medium,
};

impl Session {
    /// Sends a game packet with default settings
    /// (reliable ordered and medium priority)
    #[inline]
    pub fn send<T: ConnectedPacket + Serialize>(&self, pk: T) -> VResult<()> {
        let pk = Packet::new(pk);
        let mut serialized = pk.serialize();

        self.send_serialized(serialized, DEFAULT_SEND_CONFIG)
    }

    /// Sends a game packet with custom reliability and priority
    pub fn send_serialized(
        &self,
        mut pk: Bytes,
        config: PacketConfig,
    ) -> VResult<()> {
        let mut buffer = BytesMut::new();
        buffer.put_u8(CONNECTED_PACKET_ID);

        if self.raknet.compression_enabled.load(Ordering::SeqCst) {
            let (algorithm, threshold) = {
                let config = SERVER_CONFIG.read();
                (config.compression_algorithm, config.compression_threshold)
            };

            if pk.len() > threshold as usize {
                // Compress packet
                match SERVER_CONFIG.read().compression_algorithm {
                    CompressionAlgorithm::Snappy => {
                        unimplemented!("Snappy compression");
                    }
                    CompressionAlgorithm::Deflate => {
                        let mut writer = DeflateEncoder::new(
                            Vec::new(),
                            Compression::best(),
                        );

                        writer.write_all(pk.as_ref())?;
                        pk =
                            Bytes::copy_from_slice(writer.finish()?.as_slice());
                    }
                }
            }
        }

        if let Some(encryptor) = self.encryptor.get() {
            pk = encryptor.encrypt(pk)?;
        }
        
        buffer.put(pk);

        self.send_raw_buffer_with_config(buffer.freeze(), config);
        Ok(())
    }

    /// Sends a raw buffer with default settings
    /// (reliable ordered and medium priority).
    #[inline]
    pub fn send_raw_buffer(&self, buffer: Bytes) {
        self.send_raw_buffer_with_config(buffer, DEFAULT_SEND_CONFIG);
    }

    /// Sends a raw buffer with custom reliability and priority.
    pub fn send_raw_buffer_with_config(
        &self,
        buffer: Bytes,
        config: PacketConfig,
    ) {
        self.raknet.send_queue.insert_raw(
            config.priority,
            Frame::new(config.reliability, buffer),
        );
    }

    /// Flushes the send queue.
    pub async fn flush(&self) -> VResult<()> {
        let tick = self.current_tick.load(Ordering::SeqCst);

        if let Some(frames) = self.raknet.send_queue.flush(SendPriority::High) {
            self.send_raw_frames(frames).await?;
        }

        if tick % 2 == 0 {
            // Also flush broadcast packets.
            if let Some(frames) =
                self.raknet.send_queue.flush(SendPriority::Medium)
            {
                self.send_raw_frames(frames).await?;
            }
        }

        if tick % 4 == 0 {
            if let Some(frames) =
                self.raknet.send_queue.flush(SendPriority::Low)
            {
                self.send_raw_frames(frames).await?;
            }
        }

        // Send acknowledgements
        if tick % 4 == 0 {
            self.flush_acknowledgements().await?;
        }

        Ok(())
    }

    pub async fn flush_all(&self) -> VResult<()> {
        if let Some(frames) = self.raknet.send_queue.flush(SendPriority::High) {
            self.send_raw_frames(frames).await?;
        }

        if let Some(frames) = self.raknet.send_queue.flush(SendPriority::Medium)
        {
            self.send_raw_frames(frames).await?;
        }

        if let Some(frames) = self.raknet.send_queue.flush(SendPriority::Low) {
            self.send_raw_frames(frames).await?;
        }

        self.flush_acknowledgements().await?;
        Ok(())
    }

    pub async fn flush_acknowledgements(&self) -> VResult<()> {
        let mut confirmed = {
            let mut lock = self.raknet.confirmed_packets.lock();
            if lock.is_empty() {
                return Ok(());
            }

            let mut confirmed = Vec::new();
            std::mem::swap(lock.as_mut(), &mut confirmed);

            confirmed
        };
        confirmed.dedup();
        confirmed.sort_unstable();

        let mut records = Vec::new();
        let mut consecutive = Vec::new();
        for (index, id) in confirmed.iter().enumerate() {
            let is_last = index == confirmed.len() - 1;

            // Is range
            if !is_last && id + 1 == confirmed[index + 1] {
                consecutive.push(*id);
            } else if consecutive.is_empty() {
                records.push(AckRecord::Single(*id));
            } else {
                records.push(AckRecord::Range(consecutive[0]..*id));
                consecutive.clear();
            }
        }

        let ack = Ack { records };
        let mut serialized = BytesMut::with_capacity(ack.serialized_size());
        ack.serialize(&mut serialized);

        self.raknet
            .udp_socket
            .send_to(serialized.as_ref(), self.raknet.address)
            .await?;

        Ok(())
    }

    #[async_recursion]
    async fn send_raw_frames(&self, frames: Vec<Frame>) -> VResult<()> {
        let mut serialized = BytesMut::new();

        // Process fragments first to prevent sequence number duplication.
        for frame in &frames {
            let frame_size = frame.body.len() + std::mem::size_of::<Frame>();

            if frame_size > self.raknet.mtu as usize {
                self.raknet
                    .batch_sequence_number
                    .fetch_sub(1, Ordering::SeqCst);

                let compound = self.split_frame(frame);
                self.send_raw_frames(compound).await?;
            }
        }

        let mut batch = FrameBatch {
            sequence_number: self
                .raknet
                .batch_sequence_number
                .fetch_add(1, Ordering::SeqCst),

            frames: vec![],
        };

        let mut has_reliable_packet = false;
        for mut frame in frames {
            let frame_size = frame.body.len() + std::mem::size_of::<Frame>();

            if frame.reliability.is_ordered() {
                let order_index = self.raknet.order_channels
                    [frame.order_channel as usize]
                    .get_server_index();
                frame.order_index = order_index;
            }

            if frame.reliability.is_sequenced() {
                let sequence_index =
                    self.raknet.sequence_index.fetch_add(1, Ordering::SeqCst);
                frame.sequence_index = sequence_index;
            }

            if frame.reliability.is_reliable() {
                frame.reliable_index =
                    self.raknet.ack_index.fetch_add(1, Ordering::SeqCst);
                has_reliable_packet = true;
            }

            if batch.estimate_size() + frame_size <= self.raknet.mtu as usize {
                batch.frames.push(frame);
            } else if !batch.is_empty() {
                if has_reliable_packet {
                    self.raknet.recovery_queue.insert(batch.clone());
                }

                serialized.clear();
                batch.serialize(&mut serialized);

                // TODO: Add IPv6 support
                self.raknet
                    .udp_socket
                    .send_to(serialized.as_ref(), self.raknet.address)
                    .await?;

                has_reliable_packet = false;
                batch = FrameBatch {
                    sequence_number: self
                        .raknet
                        .batch_sequence_number
                        .fetch_add(1, Ordering::SeqCst),
                    frames: vec![frame],
                };
            }
        }

        // Send remaining packets not sent by loop
        if !batch.is_empty() {
            if has_reliable_packet {
                self.raknet.recovery_queue.insert(batch.clone());
            }

            serialized.clear();
            batch.serialize(&mut serialized);

            // TODO: Add IPv6 support
            self.raknet
                .udp_socket
                .send_to(serialized.as_ref(), self.raknet.address)
                .await?;
        } else {
            self.raknet
                .batch_sequence_number
                .fetch_sub(1, Ordering::SeqCst);
        }

        Ok(())
    }

    fn split_frame(&self, mut frame: &Frame) -> Vec<Frame> {
        let chunk_max_size = self.raknet.mtu as usize
            - std::mem::size_of::<Frame>()
            - std::mem::size_of::<FrameBatch>();
        let compound_size = {
            let frame_size = frame.body.len() + std::mem::size_of::<Frame>();

            // Ceiling divide without floating point conversion.
            // usize::div_ceil is still unstable.
            (frame_size + chunk_max_size - 1) / chunk_max_size
        };

        let mut compound = Vec::with_capacity(compound_size);
        let chunks = frame.body.chunks(chunk_max_size);

        debug_assert_eq!(chunks.len(), compound_size);

        let compound_id =
            self.raknet.compound_id.fetch_add(1, Ordering::SeqCst);

        for (i, chunk) in chunks.enumerate() {
            let mut fragment = Frame {
                reliability: frame.reliability,
                is_compound: true,
                compound_index: i as u32,
                compound_size: compound_size as u32,
                compound_id,
                body: Bytes::copy_from_slice(chunk),
                ..Default::default()
            };

            compound.push(fragment);
        }

        compound
    }
}
