use std::sync::atomic::Ordering;

use bytes::Buf;

use crate::error::VexResult;
use crate::network::raknet::frame::{Frame, FrameBatch};
use crate::network::raknet::header::Header;
use crate::network::session::send_queue::SendPriority;
use crate::network::session::session::Session;
use crate::network::traits::{Decodable, Encodable};
use crate::util::ReadExtensions;

impl Session {
    pub async fn flush_send_queue(&self, tick: u64) -> VexResult<()> {
        // TODO: Handle errors properly
        if let Some(frames) = self.send_queue.flush(SendPriority::High) {
            self.send_frames(frames).await?;
        }

        if tick % 2 == 0 {
            if let Some(frames) = self.send_queue.flush(SendPriority::Medium) {
                self.send_frames(frames).await?;
            }
        }

        if tick % 4 == 0 {
            if let Some(frames) = self.send_queue.flush(SendPriority::Low) {
                self.send_frames(frames).await?;
            }
        }

        Ok(())
    }

    async fn send_frames(&self, frames: Vec<Frame>) -> VexResult<()> {
        // TODO: Handle errors properly
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

            {
                let decoded_batch = FrameBatch::decode(encoded).unwrap();
                tracing::info!("decoded_batch {decoded_batch:?}");

                for frame in decoded_batch.get_frames() {
                    if *frame.body.first().unwrap() == 0xfe {
                        let mut frame = frame.clone();
                        frame.body.advance(1);

                        tracing::info!("{:X?}", frame.body.as_ref());
                        let length = frame.body.get_var_u32().unwrap();
                        let header = Header::decode(&mut frame.body).unwrap();
                        tracing::info!("sent {header:?}, length {length}");
                    }
                }
            }
        }

        Ok(())
    }
}
