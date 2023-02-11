use bytes::BytesMut;
use common::VResult;

use crate::network::{session::Session, packets::{TextMessage, SetTime}, Decodable};

impl Session {
    pub fn handle_text_message(&self, packet: BytesMut) -> VResult<()> {
        let request = TextMessage::decode(packet)?;
        tracing::info!("{request:?}");

        Ok(())
    }
}