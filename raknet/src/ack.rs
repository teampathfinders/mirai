use util::{Deserialize, BinaryRead, Serialize};

use proto::raknet::{Ack, Nak};

use crate::RaknetUser;

impl RaknetUser {
    /// Processes an acknowledgement received from the client.
    ///
    /// This function unregisters the specified packet IDs from the recovery queue.
    pub fn handle_ack<'a, R: BinaryRead<'a>>(&self, reader: R) -> anyhow::Result<()> {
        let ack = Ack::deserialize(reader)?;

        #[cfg(trace_raknet)]
        tracing::debug!("{ack:?}");

        self.recovery.acknowledge(&ack.records);

        Ok(())
    }

    /// Processes a negative acknowledgement received from the client.
    ///
    /// This function makes sure the packet is retrieved from the recovery queue and sent to the
    /// client again.
    #[allow(clippy::future_not_send)]
    pub async fn handle_nack<'a, R: BinaryRead<'a>>(&self, reader: R) -> anyhow::Result<()> {
        let nack = Nak::deserialize(reader)?;
        let frame_batches = self.recovery.recover(&nack.records);

        let mut serialized = Vec::new();
        for frame_batch in frame_batches {
            frame_batch.serialize_into(&mut serialized)?;

            self
                .socket
                .send_to(
                    serialized.as_ref(), self.address,
                )
                .await?;

            serialized.clear();
        }

        Ok(())
    }
}
