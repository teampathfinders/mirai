use bytes::BytesMut;
use common::VResult;

use crate::network::{packets::Interact, session::Session, Decodable};

impl Session {
    pub fn handle_interaction(&self, packet: BytesMut) -> VResult<()> {
        tracing::info!("{:x?}", packet.as_ref());

        let request = Interact::decode(packet)?;
        tracing::info!("{request:?}");

        Ok(())
    }
}
