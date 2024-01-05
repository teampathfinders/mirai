use std::io::Write;
use std::sync::atomic::Ordering;

use async_recursion::async_recursion;
use flate2::Compression;
use flate2::write::DeflateEncoder;
use proto::bedrock::{CompressionAlgorithm, CONNECTED_PACKET_ID, ConnectedPacket, Packet};
use proto::raknet::{Ack, AckRecord};

use util::{BinaryWrite, MutableBuffer};

use util::Serialize;

use crate::raknet::{Frame, FrameBatch};
use crate::raknet::Reliability;
use crate::raknet::SendPriority;
use crate::config::SERVER_CONFIG;
use crate::network::Session;

/// Specifies the reliability and priority of a packet.
pub struct PacketConfig {
    /// In case encryption is enabled, this reliability must always be reliable ordered.
    pub reliability: Reliability,
    /// Priority specifies if this packet has sending priority over other raknet.
    pub priority: SendPriority,
}

/// A default packet config that can be used for all raknet.
pub const DEFAULT_SEND_CONFIG: PacketConfig = PacketConfig {
    reliability: Reliability::ReliableOrdered,
    priority: SendPriority::Medium,
};

impl Session {
    /// Sends a game packet with default settings
    /// (reliable ordered and medium priority)
    #[inline]
    pub fn send<T: ConnectedPacket + Serialize>(&self, packet: T) -> anyhow::Result<()> {
        let packet = Packet::new(packet);
        let serialized = packet.serialize()?;

        self.send_serialized(serialized, DEFAULT_SEND_CONFIG)
    }

    /// Sends a game packet with custom reliability and priority
    pub fn send_serialized<B>(&self, packet: B, config: PacketConfig) -> anyhow::Result<()>
        where
            B: AsRef<[u8]>
    {
        let mut out;
        if self.raknet.compression_enabled.load(Ordering::SeqCst) {
            let (algorithm, threshold) = {
                let config = SERVER_CONFIG.read();
                (config.compression_algorithm, config.compression_threshold)
            };

            if packet.as_ref().len() > threshold as usize {
                // Compress packet
                match algorithm {
                    CompressionAlgorithm::Snappy => {
                        unimplemented!("Snappy compression");
                    }
                    CompressionAlgorithm::Deflate => {
                        let mut writer = DeflateEncoder::new(
                            vec![CONNECTED_PACKET_ID],
                            Compression::best(),
                        );

                        writer.write_all(packet.as_ref())?;
                        out = MutableBuffer::from(writer.finish()?)
                    }
                }
            } else {
                // Also reserve capacity for checksum even if encryption is disabled,
                // preventing allocations.
                out = MutableBuffer::with_capacity(1 + packet.as_ref().len() + 8);
                out.write_u8(CONNECTED_PACKET_ID)?;
                out.write_all(packet.as_ref())?;
            }
        } else {
            // Also reserve capacity for checksum even if encryption is disabled,
            // preventing allocations.
            out = MutableBuffer::with_capacity(1 + packet.as_ref().len() + 8);
            out.write_u8(CONNECTED_PACKET_ID)?;
            out.write_all(packet.as_ref())?;
        };

        if let Some(encryptor) = self.encryptor.get() {
            encryptor.encrypt(&mut out)?;
        };

        self.send_raw_buffer_with_config(out, config);
        Ok(())
    }

    /// Sends a raw buffer with default settings
    /// (reliable ordered and medium priority).
    #[inline]
    pub fn send_raw_buffer<B>(&self, buffer: B)
        where
            B: Into<MutableBuffer>
    {
        self.send_raw_buffer_with_config(buffer, DEFAULT_SEND_CONFIG);
    }

    /// Sends a raw buffer with custom reliability and priority.
    pub fn send_raw_buffer_with_config<B>(
        &self,
        buffer: B,
        config: PacketConfig,
    ) where B: Into<MutableBuffer> {
        let buffer = buffer.into();
        self.raknet.send_queue.insert_raw(
            config.priority,
            Frame::new(config.reliability, buffer),
        );
    }

    /// Flushes the send queue.
    pub async fn flush(&self) -> anyhow::Result<()> {
        let tick = self.current_tick.load(Ordering::SeqCst);

        if let Some(frames) = self.raknet.send_queue.flush(SendPriority::High) {
            self.send_raw_frames(frames).await?;
        }

        if tick % 2 == 0 {
            // Also flush broadcast raknet.
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

    pub async fn flush_all(&self) -> anyhow::Result<()> {
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

    pub async fn flush_acknowledgements(&self) -> anyhow::Result<()> {
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
        let mut serialized = MutableBuffer::with_capacity(ack.serialized_size());
        ack.serialize(&mut serialized)?;

        self.raknet
            .udp_socket
            .send_to(serialized.as_ref(), self.raknet.address)
            .await?;

        Ok(())
    }

    #[async_recursion]
    async fn send_raw_frames(&self, mut frames: Vec<Frame>) -> anyhow::Result<()> {
        let mut serialized = MutableBuffer::new();

        // Process fragments first to prevent sequence number duplication.
        let mut index = 0;
        while index < frames.len() {
            let frame_size = frames[index].body.len() + std::mem::size_of::<Frame>();

            if frame_size > self.raknet.mtu as usize {
                self.raknet
                    .batch_sequence_number
                    .fetch_sub(1, Ordering::SeqCst);

                let large_frame = frames.swap_remove(index);
                let compound = self.split_frame(&large_frame);
                self.send_raw_frames(compound).await?;
            } else {
                index += 1;
            }
        }

        debug_assert!(
            frames
                .iter()
                .find(|f| f.body.len() > self.raknet.mtu as usize - std::mem::size_of::<Frame>())
                .is_none(),
            "Frames were not split properly"
        );

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
                    .fetch_index();
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
                serialized.clear();
                batch.serialize(&mut serialized)?;

                // TODO: Add IPv6 support
                self.raknet
                    .udp_socket
                    .send_to(serialized.as_ref(), self.raknet.address)
                    .await?;

                if has_reliable_packet {
                    self.raknet.recovery_queue.insert(batch);
                }

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

        // Send remaining raknet not sent by loop
        if !batch.is_empty() {
            serialized.clear();
            batch.serialize(&mut serialized)?;

            if has_reliable_packet {
                self.raknet.recovery_queue.insert(batch);
            }

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

    fn split_frame(&self, frame: &Frame) -> Vec<Frame> {
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
            let fragment = Frame {
                reliability: frame.reliability,
                is_compound: true,
                compound_index: i as u32,
                compound_size: compound_size as u32,
                compound_id,
                body: MutableBuffer::from(chunk.to_vec()),
                ..Default::default()
            };

            compound.push(fragment);
        }

        compound
    }
}
