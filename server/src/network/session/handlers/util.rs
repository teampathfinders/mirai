use bytes::BytesMut;
use common::{VResult, Vector3i, Vector3f};

use crate::network::{
    packets::{
        CreditStatus, Difficulty, GameMode, MessageType, MobEffectKind, MobEffectOperation,
        MobEffectUpdate, PlaySound, SetDifficulty, SetPlayerGameMode, SetTime,
        ShowCredits, ShowProfile, TextMessage, SetCommandsEnabled, AddPainting, PaintingDirection, ChangeDimension, Dimension,
    },
    session::Session,
    Decodable,
};

impl Session {
    pub fn handle_text_message(&self, packet: BytesMut) -> VResult<()> {
        let request = TextMessage::decode(packet)?;
        tracing::info!("{request:?}");

        let reply = ChangeDimension {
            dimension: Dimension::Nether,
            position: Vector3f::from([0.0, 0.0, 0.0]),
            respawn: false
        };
        self.send_packet(reply)?;

        Ok(())
    }
}
