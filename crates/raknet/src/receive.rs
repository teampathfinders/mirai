
use std::sync::atomic::Ordering;
use std::time::{Instant, Duration};

use async_recursion::async_recursion;
use proto::bedrock::CONNECTED_PACKET_ID;
use proto::raknet::{Ack, ConnectedPing, ConnectionRequest, DisconnectNotification, Nak, NewIncomingConnection};
use util::{RVec, Deserialize};

use tokio::sync::mpsc::error::SendTimeoutError;

use crate::{Frame, FrameBatch, RakNetCommand, RakNetClient};

const RAKNET_OUTPUT_TIMEOUT: Duration = Duration::from_millis(10);

impl RakNetClient {
    /// Processes the raw packet coming directly from the network.
    ///
    /// If a packet is an ACK or NACK type, it will be responded to accordingly (using [`Session::process_ack`] and [`Session::process_nak`]).
    /// Frame batches are processed by [`Session::process_frame_batch`].
    #[tracing::instrument(
        skip_all,
        name = "RaknetUser::handle_raw_packet",
        fields(
            address = %self.address
        )
    )]
    pub async fn handle_raw_packet(&self, packet: RVec) -> anyhow::Result<bool> {
        *self.last_update.write() = Instant::now();

        let Some(pk_id) = packet.first().copied() else {
            tracing::warn!("Received raw packet is empty");
            anyhow::bail!("Raw packet is empty");
        };  

        match pk_id {
            Ack::ID => self.handle_ack(packet.as_ref())?,
            Nak::ID => self.handle_nak(packet.as_ref()).await?,
            _ => self.handle_frame_batch(packet).await?,
        }

        Ok(true)
    }

    /// Processes a batch of frames.
    ///
    /// This performs the actions required by the Raknet reliability layer, such as
    /// * Inserting raknet into the order channels
    /// * Inserting raknet into the compound collector
    /// * Discarding old sequenced frames
    /// * Acknowledging reliable raknet
    async fn handle_frame_batch(&self, packet: RVec) -> anyhow::Result<()> {
        let batch = FrameBatch::deserialize(packet.as_ref())?;
        // self
        //     .batch_number
        //     .fetch_max(batch.sequence_number, Ordering::SeqCst);

        for frame in batch.frames {
            self.handle_frame(frame, batch.sequence_number).await?;
        }

        Ok(())
    }

    #[async_recursion]
    async fn handle_frame(
        &self,
        frame: Frame,
        batch_number: u32,
    ) -> anyhow::Result<()> {
        if frame.reliability.is_sequenced()
            && frame.sequence_index
            < self.batch_number.load(Ordering::SeqCst)
        {
            tracing::warn!("Received old packet. Discarding it");

            // Discard packet
            return Ok(());
        }

        if frame.reliability.is_reliable() {
            // Confirm packet
            let mut lock = self.acknowledged.lock();
            lock.push(batch_number);
        }

        if frame.is_compound {
            let possible_frag = self.compounds.insert(frame)?;

            return if let Some(packet) = possible_frag {
                self.handle_frame(packet, batch_number).await
            } else {
                // Compound incomplete
                Ok(())
            };
        }

        // Sequenced implies ordered
        if frame.reliability.is_ordered() || frame.reliability.is_sequenced() {
            // Add packet to order queue
            if let Ok(ready) = self.order[frame.order_channel as usize]
                .insert(frame)
            {
                if let Some(ready) = ready {
                    for packet in ready {
                        self.handle_frame_body(packet.body).await?;
                    }
                }
            } else {
                tracing::error!("Failed to insert packet into order channel");
                anyhow::bail!("Failed to insert packet into order channel");
            }

            return Ok(());
        }

        self.handle_frame_body(frame.body).await
    }

    /// Processes an unencapsulated game packet.
    async fn handle_frame_body(&self, packet: RVec) -> anyhow::Result<()> {
        let Some(packet_id) = packet.first().copied() else {
            tracing::warn!("Received packet is empty");
            anyhow::bail!("Packet was empty");
        };

        match packet_id {
            // CONNECTED_PACKET_ID => self.handle_encrypted_frame(packet).await?,
            CONNECTED_PACKET_ID => {
                if let Err(err) = self.output.send_timeout(RakNetCommand::Received(packet), RAKNET_OUTPUT_TIMEOUT).await {
                    if matches!(err, SendTimeoutError::Closed(_)) {
                        // Output channel has been closed
                        tracing::warn!("RakNet layer output channel closed, disconnecting them...");
                    } else {
                        // Forward timeout
                        tracing::warn!("Client seems to be hanging server side, disconnecting them...")
                    }
                    self.disconnect();
                }
            },
            DisconnectNotification::ID => self.active.cancel(),
            ConnectionRequest::ID => self.handle_connection_request(packet)?,
            NewIncomingConnection::ID => {
                self.handle_new_incoming_connection(packet)?
            }
            ConnectedPing::ID => self.handle_connected_ping(packet)?,
            id => anyhow::bail!("Invalid Raknet packet ID: {}", id),
        }

        Ok(())
    }
}