use std::sync::atomic::Ordering;
use std::time::Instant;

use async_recursion::async_recursion;
use bytes::BytesMut;

use vex_common::{bail, Decodable, VResult};

use crate::frame::{Frame, FrameBatch};
use crate::packets::{Acknowledgement, ConnectionRequest, DisconnectNotification, NegativeAcknowledgement, NewIncomingConnection, OnlinePing};
use crate::session::{GAME_PACKET_ID, Session};

impl Session {
    /// Processes the raw packet coming directly from the network.
    ///
    /// If a packet is an ACK or NACK type, it will be responded to accordingly (using [`Session::handle_ack`] and [`Session::handle_nack`]).
    /// Frame batches are processed by [`Session::handle_frame_batch`].
    pub async fn handle_raw_packet(&self) -> VResult<()> {
        let packet = tokio::select! {
            _ = self.token.cancelled() => {
                return Ok(())
            },
            task = self.receive_queue.pop() => task
        };
        *self.last_update.write() = Instant::now();

        if packet.is_empty() {
            bail!(BadPacket, "Packet is empty");
        }

        let packet_id = *packet.first().unwrap();
        match packet_id {
            Acknowledgement::ID => self.handle_ack(packet),
            NegativeAcknowledgement::ID => self.handle_nack(packet).await,
            _ => self.handle_frame_batch(packet).await,
        }
    }

    /// Processes a batch of frames.
    ///
    /// This performs the actions required by the Raknet reliability layer, such as
    /// * Inserting packets into the order channels
    /// * Inserting packets into the compound collector
    /// * Discarding old sequenced frames
    /// * Acknowledging reliable packets
    async fn handle_frame_batch(&self, task: BytesMut) -> VResult<()> {
        let batch = FrameBatch::decode(task)?;
        self.client_batch_number
            .fetch_max(batch.get_batch_number(), Ordering::SeqCst);

        for frame in batch.get_frames() {
            self.handle_frame(frame, batch.get_batch_number()).await?;
        }

        Ok(())
    }

    #[async_recursion]
    async fn handle_frame(&self, frame: &Frame, batch_number: u32) -> VResult<()> {
        if frame.reliability.is_sequenced()
            && frame.sequence_index < self.client_batch_number.load(Ordering::SeqCst)
        {
            // Discard packet
            return Ok(());
        }

        if frame.reliability.is_reliable() {
            // Confirm packet
            let mut lock = self.confirmed_packets.lock();
            lock.push(batch_number);
        }

        if frame.is_compound {
            if let Some(p) = self.compound_collector.insert(frame.clone()) {
                return self.handle_frame(&p, batch_number).await;
            }

            return Ok(());
        }

        // TODO: Handle errors in processing properly

        // Sequenced implies ordered
        if frame.reliability.is_ordered() || frame.reliability.is_sequenced() {
            // Add packet to order queue
            if let Some(ready) =
                self.order_channels[frame.order_channel as usize].insert(frame.clone())
            {
                for packet in ready {
                    self.handle_unframed_packet(packet.body).await?;
                }
            }
            return Ok(());
        }

        self.handle_unframed_packet(frame.body.clone()).await?;
        Ok(())
    }

    /// Processes an unencapsulated game packet.
    async fn handle_unframed_packet(&self, mut task: BytesMut) -> VResult<()> {
        let bytes = task.as_ref();

        let packet_id = *task.first().expect("Game packet buffer was empty");
        match packet_id {
            GAME_PACKET_ID => self.handle_game_packet(task).await?,
            DisconnectNotification::ID => self.flag_for_close(),
            ConnectionRequest::ID => self.handle_connection_request(task)?,
            NewIncomingConnection::ID => self.handle_new_incoming_connection(task)?,
            OnlinePing::ID => self.handle_online_ping(task)?,
            id => bail!(BadPacket, "Invalid Raknet packet ID: {}", id),
        }

        Ok(())
    }
}
