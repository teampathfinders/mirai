use bytes::BytesMut;
use common::{VResult, Vector3i};

use crate::network::{
    packets::{
        GameMode, MessageType, MobEffectKind, MobEffectOperation, MobEffectUpdate, PlaySound,
        SetHealth, SetPlayerGameMode, SetTime, ShowProfile, TextMessage, ShowCredits, CreditStatus, SetDifficulty, Difficulty,
    },
    session::Session,
    Decodable,
};

impl Session {
    pub fn handle_text_message(&self, packet: BytesMut) -> VResult<()> {
        let request = TextMessage::decode(packet)?;
        tracing::info!("{request:?}");

        let reply = SetDifficulty {
            difficulty: Difficulty::Peaceful
        };
        self.send_packet(reply)?;

        Ok(())
    }
}
