use std::sync::atomic::Ordering;

use async_recursion::async_recursion;
use proto::raknet::{Ack, AckEntry};

use util::{RVec, Serialize};

use crate::{SendPriority, RakNetClient, Reliability, Frame, FrameBatch};

/// Specifies the reliability and priority of a packet.
pub struct SendConfig {
    /// In case encryption is enabled, this reliability must always be reliable ordered.
    pub reliability: Reliability,
    /// Priority specifies if this packet has sending priority over other raknet.
    pub priority: SendPriority,
}

/// A default packet config that can be used for all raknet.
pub const DEFAULT_SEND_CONFIG: SendConfig = SendConfig {
    reliability: Reliability::ReliableOrdered,
    priority: SendPriority::Medium,
};

impl RakNetClient {
    /// Sends a raw buffer with default settings
    /// (reliable ordered and medium priority).
    pub fn send_raw_buffer<B>(&self, buffer: B)
        where
            B: Into<RVec>
    {
        self.send_raw_buffer_with_config(buffer, DEFAULT_SEND_CONFIG);
    }

    /// Sends a raw buffer with custom reliability and priority.
    pub fn send_raw_buffer_with_config<B>(
        &self,
        buffer: B,
        config: SendConfig,
    ) where B: Into<RVec> {
        let buffer = buffer.into();
        self.send.insert_raw(
            config.priority,
            Frame::new(config.reliability, buffer),
        );
    }

    /// Flushes the send queue.
    pub async fn flush(&self) -> anyhow::Result<()> {
        let tick = self.tick.load(Ordering::SeqCst);

        if let Some(frames) = self.send.flush(SendPriority::High) {
            self.send_raw_frames(frames).await?;
        }

        if tick % 2 == 0 {
            // Also flush broadcast raknet.
            if let Some(frames) =
                self.send.flush(SendPriority::Medium)
            {
                self.send_raw_frames(frames).await?;
            }
        }

        if tick % 4 == 0 {
            if let Some(frames) =
                self.send.flush(SendPriority::Low)
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

    /// Flushes both the frames and acknowledgements.
    pub async fn flush_all(&self) -> anyhow::Result<()> {
        if let Some(frames) = self.send.flush(SendPriority::High) {
            self.send_raw_frames(frames).await?;
        }

        if let Some(frames) = self.send.flush(SendPriority::Medium)
        {
            self.send_raw_frames(frames).await?;
        }

        if let Some(frames) = self.send.flush(SendPriority::Low) {
            self.send_raw_frames(frames).await?;
        }

        self.flush_acknowledgements().await
    }

    /// Flushes all of the pending acknowledgements.
    pub async fn flush_acknowledgements(&self) -> anyhow::Result<()> {
        let mut confirmed = {
            let mut lock = self.acknowledged.lock();
            if lock.is_empty() {
                return Ok(());
            }

            // Usually no more than 5 packets will be acknowledged at a time.
            // Reserving 5 elements will prevent reallocations.
            let mut confirmed = Vec::with_capacity(5);
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
                records.push(AckEntry::Single(*id));
            } else {
                records.push(AckEntry::Range(consecutive[0]..*id));
                consecutive.clear();
            }
        }

        let ack = Ack { records };
        let mut serialized = RVec::alloc_with_capacity(ack.serialized_size());
        ack.serialize_into(&mut serialized)?;

        self
            .socket
            .send_to(serialized.as_ref(), self.address)
            .await?;

        Ok(())
    }

    /// Send a list of frames. 
    ///
    /// These frames are not guaranteed to be sent in the same frame batch.
    /// If the batch would be bigger than the MTU, the list will be split into multiple batches.
    /// Similarly if a frame is larger than the MTU, it will be split and sent in multiple packets.
    ///
    /// ## Warning 
    ///
    /// In case the passed frames are already fragmented, there should at maximum one compound
    /// in the entire list.
    #[async_recursion]
    async fn send_raw_frames(&self, mut frames: Vec<Frame>) -> anyhow::Result<()> {
        let mut serialized = Vec::new();

        // Process fragments first to prevent sequence number duplication.
        let mut index = 0;
        while index < frames.len() {
            let frame_size = frames[index].body.len() + std::mem::size_of::<Frame>();

            if frame_size > self.mtu as usize {
                let large_frame = frames.swap_remove(index);
                let compound = self.split_frame(&large_frame);

                // Only one compound is passed to the function, so this is fine.
                self.send_raw_frames(compound).await?;
            } else {
                index += 1;
            }
        }
        
        debug_assert!(
            !frames
                .iter()
                .any(|f| f.body.len() > self.mtu as usize - std::mem::size_of::<Frame>()),
            "Frames were not split properly"
        );

        let mut batch = FrameBatch {
            sequence_number: 0,
            frames: vec![],
        };

        let mut has_reliable_packet = false;    

        // Set to u32::MAX when unset, otherwise set to the compound's order index
        let mut compound_order_index = u32::MAX;

        for mut frame in frames {
            let frame_size = frame.body.len() + std::mem::size_of::<Frame>();

            if frame.reliability.is_ordered() && !frame.is_compound {
                let order_index = self.order[frame.order_channel as usize]
                    .alloc_index();

                frame.order_index = order_index;
            } else if frame.reliability.is_ordered() {
                if compound_order_index == u32::MAX {
                    let new_index = self.order[0].alloc_index();
                    compound_order_index = new_index;
                }

                frame.order_channel = 0;
                frame.order_index = compound_order_index;
            }

            if frame.reliability.is_sequenced() {
                let sequence_index =
                    self.sequence_index.fetch_add(1, Ordering::SeqCst);

                frame.sequence_index = sequence_index;
            }

            if frame.reliability.is_reliable() {
                frame.reliable_index =
                    self.acknowledge_index.fetch_add(1, Ordering::SeqCst);

                has_reliable_packet = true;
            }

            #[allow(clippy::unwrap_used)]
            if batch.size_hint().unwrap() + frame_size <= self.mtu as usize {
                batch.frames.push(frame);
            } else if !batch.is_empty() {
                serialized.clear();

                batch.sequence_number = self.batch_number.fetch_add(1, Ordering::SeqCst);
                batch.serialize_into(&mut serialized)?;

                // TODO: Add IPv6 support
                self.socket
                    .send_to(serialized.as_ref(), self.address)
                    .await?;

                if has_reliable_packet {
                    self.recovery.insert(batch);
                }

                has_reliable_packet = false;
                batch = FrameBatch {
                    sequence_number: self
                        .batch_number
                        .fetch_add(0, Ordering::SeqCst),

                    frames: vec![frame],
                };
            }
        }

        // Send remaining packets not sent by loop
        if !batch.is_empty() {
            serialized.clear();

            batch.sequence_number = self.batch_number.fetch_add(1, Ordering::SeqCst);
            batch.serialize_into(&mut serialized)?;

            if has_reliable_packet {
                self.recovery.insert(batch);
            }

            // TODO: Add IPv6 support
            self.socket
                .send_to(serialized.as_ref(), self.address)
                .await?;
        }
        // } else {
        //     self.batch_number.fetch_sub(1, Ordering::SeqCst);
        // }

        Ok(())
    }

    fn split_frame(&self, frame: &Frame) -> Vec<Frame> {
        println!("{}", frame.body.len());

        let chunk_max_size = self.mtu as usize
            - std::mem::size_of::<Frame>()
            - std::mem::size_of::<FrameBatch>();

        let compound_size = frame.body.len().div_ceil(chunk_max_size);
        let mut compound = Vec::with_capacity(compound_size);
        let chunks = frame.body.chunks(chunk_max_size);

        debug_assert_eq!(chunks.len(), compound_size, "Chunk count does not match compound size");

        let compound_id =
            self.compound_id.fetch_add(1, Ordering::SeqCst);

        for (i, chunk) in chunks.enumerate() {
            let fragment = Frame {
                reliability: frame.reliability,
                is_compound: true,
                compound_index: i as u32,
                compound_size: compound_size as u32,
                compound_id,
                body: RVec::alloc_from_slice(chunk),                
                ..Default::default()
            };

            compound.push(fragment);
        }

        compound
    }
}