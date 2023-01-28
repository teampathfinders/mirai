use bytes::BytesMut;

use crate::error::VexResult;
use crate::network::raknet::packets::acknowledgements::{Ack, Nack};
use crate::network::session::session::Session;
use crate::network::traits::{Decodable, Encodable};

impl Session {
    /// Processes an acknowledgement received from the client.
    ///
    /// This function unregisters the specified packet IDs from the recovery queue.
    pub fn handle_ack(&self, task: BytesMut) -> VexResult<()> {
        let ack = Ack::decode(task)?;
        self.recovery_queue.confirm(&ack.records);

        Ok(())
    }

    /// Processes a negative acknowledgement received from the client.
    ///
    /// This function makes sure the packet is retrieved from the recovery queue and sent to the
    /// client again.
    pub async fn handle_nack(&self, task: BytesMut) -> VexResult<()> {
        let nack = Nack::decode(task)?;
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
