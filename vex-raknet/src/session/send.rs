use std::sync::atomic::Ordering;

use bytes::BytesMut;

use vex_common::{Encodable, VResult};

use crate::frame::{Frame, FrameBatch};
use crate::packets::{Acknowledgement, AcknowledgementRecord};
use crate::reliability::Reliability;
use crate::session::{SendPriority, Session};

pub struct PacketConfig {
    pub reliability: Reliability,
    pub priority: SendPriority,
}

const DEFAULT_CONFIG: PacketConfig = PacketConfig {
    reliability: Reliability::ReliableOrdered,
    priority: SendPriority::Medium,
};

impl Session {
    /// Sends a raw buffer with default settings
    /// (reliable ordered and medium priority).
    pub fn send_raw_buffer(&self, buffer: BytesMut) {
        self.send_raw_buffer_with_config(buffer, DEFAULT_CONFIG);
    }

    /// Sends a raw buffer with custom reliability and priority.
    pub fn send_raw_buffer_with_config(&self, buffer: BytesMut, config: PacketConfig) {
        self.send_queue
            .insert_raw(config.priority, Frame::new(config.reliability, buffer));
    }

    /// Flushes the send queue.
    pub async fn flush(&self) -> VResult<()> {
        let tick = self.current_tick.load(Ordering::SeqCst);

        if let Some(frames) = self.send_queue.flush(SendPriority::High) {
            self.send_raw_frames(frames).await?;
        }

        if tick % 2 == 0 {
            if let Some(frames) = self.send_queue.flush(SendPriority::Medium) {
                self.send_raw_frames(frames).await?;
            }
        }

        if tick % 4 == 0 {
            if let Some(frames) = self.send_queue.flush(SendPriority::Low) {
                self.send_raw_frames(frames).await?;
            }
        }

        // Send acknowledgements
        if tick % 20 == 0 {
            self.flush_acknowledgements().await?;
        }

        Ok(())
    }

    pub async fn flush_acknowledgements(&self) -> VResult<()> {
        let mut confirmed = {
            let mut lock = self.confirmed_packets.lock();
            if lock.is_empty() {
                return Ok(());
            }

            let mut confirmed = Vec::new();
            std::mem::swap(lock.as_mut(), &mut confirmed);

            confirmed
        };
        confirmed.dedup();

        let mut records = Vec::new();
        let mut consecutive = Vec::new();
        for (index, id) in confirmed.iter().enumerate() {
            let is_last = index == confirmed.len() - 1;

            // Is range
            if !is_last && id + 1 == confirmed[index + 1] {
                consecutive.push(*id);
            } else if consecutive.is_empty() {
                records.push(AcknowledgementRecord::Single(*id));
            } else {
                records.push(AcknowledgementRecord::Range(consecutive[0]..*id));
                consecutive.clear();
            }
        }

        let ack = Acknowledgement { records }.encode()?;
        self.ipv4_socket.send_to(&ack, self.address).await?;

        Ok(())
    }

    async fn send_raw_frames(&self, frames: Vec<Frame>) -> VResult<()> {
        let max_batch_size = self.mtu as usize - std::mem::size_of::<FrameBatch>();
        let mut batch =
            FrameBatch::default().batch_number(self.batch_number.fetch_add(1, Ordering::SeqCst));

        let mut has_reliable_packet = false;

        for mut frame in frames {
            let frame_size = frame.body.len() + std::mem::size_of::<Frame>();

            if frame_size > self.mtu as usize {
                todo!("Create compound");
            }
            if frame.reliability.is_ordered() {
                let order_index =
                    self.order_channels[frame.order_channel as usize].get_server_index();
                frame.order_index = order_index;
            }
            if frame.reliability.is_sequenced() {
                let sequence_index = self.sequence_index.fetch_add(1, Ordering::SeqCst);
                frame.sequence_index = sequence_index;
            }
            if frame.reliability.is_reliable() {
                frame.reliable_index = self.acknowledgment_index.fetch_add(1, Ordering::SeqCst);
                has_reliable_packet = true;
            }

            if batch.estimate_size() + frame_size < max_batch_size {
                batch = batch.push(frame);
            } else {
                if has_reliable_packet {
                    self.recovery_queue.insert(batch.clone());
                }
                let encoded = batch.encode()?;

                // TODO: Add IPv6 support
                self.ipv4_socket.send_to(&encoded, self.address).await?;

                has_reliable_packet = false;
                batch = FrameBatch::default()
                    .batch_number(self.batch_number.fetch_add(1, Ordering::SeqCst));
            }
        }

        // Send remaining packets not sent by loop
        if !batch.is_empty() {
            if has_reliable_packet {
                self.recovery_queue.insert(batch.clone());
            }
            let encoded = batch.encode()?;

            // TODO: Add IPv6 support
            self.ipv4_socket.send_to(&encoded, self.address).await?;
        }

        Ok(())
    }
}
