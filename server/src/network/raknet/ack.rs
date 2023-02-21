use bytes::{Bytes, BytesMut};

use crate::network::raknet::packets::{Ack, Nak};
use crate::network::session::Session;
use common::VResult;
use common::{Deserialize, Serialize};

impl Session {
    /// Processes an acknowledgement received from the client.
    ///
    /// This function unregisters the specified packet IDs from the recovery queue.
    pub fn handle_ack(&self, pk: Bytes) -> VResult<()> {
        let ack = Ack::deserialize(pk)?;
        self.raknet.recovery_queue.confirm(&ack.records);

        Ok(())
    }

    /// Processes a negative acknowledgement received from the client.
    ///
    /// This function makes sure the packet is retrieved from the recovery queue and sent to the
    /// client again.
    pub async fn handle_nack(&self, pk: Bytes) -> VResult<()> {
        let nack = Nak::deserialize(pk)?;
        let frame_batches = self.raknet.recovery_queue.recover(&nack.records);
        tracing::info!("Recovered packets: {:?}", nack.records);

        let mut serialized = BytesMut::new();
        for frame_batch in frame_batches {
            frame_batch.serialize(&mut serialized);            

            self.raknet
                .udp_socket
                .send_to(
                    serialized.as_ref(), self.raknet.address
                )
                .await?;

            serialized.clear();
        }

        Ok(())
    }
}
