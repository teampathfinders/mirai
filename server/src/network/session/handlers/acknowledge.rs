use bytes::BytesMut;

use crate::network::raknet::packets::{Acknowledgement, NegativeAcknowledgement};
use crate::network::session::session::Session;
use common::VResult;
use common::{Decodable, Encodable};

impl Session {
    /// Processes an acknowledgement received from the client.
    ///
    /// This function unregisters the specified packet IDs from the recovery queue.
    pub fn handle_ack(&self, task: BytesMut) -> VResult<()> {
        let ack = Acknowledgement::decode(task)?;
        self.recovery_queue.confirm(&ack.records);

        Ok(())
    }

    /// Processes a negative acknowledgement received from the client.
    ///
    /// This function makes sure the packet is retrieved from the recovery queue and sent to the
    /// client again.
    pub async fn handle_nack(&self, task: BytesMut) -> VResult<()> {
        let nack = NegativeAcknowledgement::decode(task)?;
        let frame_batches = self.recovery_queue.recover(&nack.records);
        tracing::info!("Recovered packets: {:?}", nack.records);

        for frame_batch in frame_batches {
            self.ipv4_socket
                .send_to(frame_batch.encode()?.as_ref(), self.address)
                .await?;
        }

        Ok(())
    }
}
