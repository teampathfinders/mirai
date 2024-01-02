use util::bytes::{MutableBuffer, SharedBuffer};
use util::{Deserialize, Result};

use crate::raknet::{Ack, Nak};
use crate::network::Session;

impl Session {
    /// Processes an acknowledgement received from the client.
    ///
    /// This function unregisters the specified packet IDs from the recovery queue.
    pub fn process_ack(&self, packet: SharedBuffer<'_>) -> anyhow::Result<()> {
        let ack = Ack::deserialize(packet)?;
        self.raknet.recovery_queue.confirm(&ack.records);

        Ok(())
    }

    /// Processes a negative acknowledgement received from the client.
    ///
    /// This function makes sure the packet is retrieved from the recovery queue and sent to the
    /// client again.
    pub async fn process_nak(&self, packet: SharedBuffer<'_>) -> anyhow::Result<()> {
        let nack = Nak::deserialize(packet)?;
        let frame_batches = self.raknet.recovery_queue.recover(&nack.records);
        tracing::info!("Recovered raknet: {:?}", nack.records);

        let mut serialized = MutableBuffer::new();
        for frame_batch in frame_batches {
            frame_batch.serialize(&mut serialized)?;

            self.raknet
                .udp_socket
                .send_to(
                    serialized.as_ref(), self.raknet.address,
                )
                .await?;

            serialized.clear();
        }

        Ok(())
    }
}
