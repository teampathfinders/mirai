use util::bytes::{MutableBuffer, SharedBuffer};
use util::{Deserialize, Result};

use crate::raknet::{Ack, Nak};
use crate::network::Session;

use super::RakNetSession;

impl RakNetSession {
    /// Processes an acknowledgement received from the client.
    ///
    /// This function unregisters the specified packet IDs from the recovery queue.
    pub fn process_ack(&self, packet: SharedBuffer<'_>) -> anyhow::Result<()> {
        let ack = Ack::deserialize(packet)?;
        self.recovery_queue.confirm(&ack.records);

        Ok(())
    }

    /// Processes a negative acknowledgement received from the client.
    ///
    /// This function makes sure the packet is retrieved from the recovery queue and sent to the
    /// client again.
    pub async fn process_nak(&self, packet: SharedBuffer<'_>) -> anyhow::Result<()> {
        let nack = Nak::deserialize(packet)?;
        let frame_batches = self.recovery_queue.recover(&nack.records);
        tracing::info!("Recovered packets: {:?}", nack.records);

        let mut serialized = MutableBuffer::new();
        for frame_batch in frame_batches {
            frame_batch.serialize(&mut serialized)?;

            self
                .udp_controller
                .send_to(
                    serialized.as_ref(), self.addr,
                )
                .await?;

            serialized.clear();
        }

        Ok(())
    }
}
