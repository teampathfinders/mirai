use bytes::BytesMut;
use common::{VResult, Vector3i, Vector3f};

use crate::network::{
    packets::{
        CreditStatus, Difficulty, GameMode, MessageType, MobEffectKind, MobEffectOperation,
        MobEffectUpdate, PlaySound, SetDifficulty, SetHealth, SetPlayerGameMode, SetTime,
        ShowCredits, ShowProfile, TextMessage, SetCommandsEnabled, AddPainting, PaintingDirection,
    },
    session::Session,
    Decodable,
};

impl Session {
    pub fn handle_text_message(&self, packet: BytesMut) -> VResult<()> {
        let request = TextMessage::decode(packet)?;
        tracing::info!("{request:?}");

        let reply = AddPainting {
            name: "BurningSkull".to_owned(),
            position: Vector3f::from([1.0, 3.0, 4.0]),
            runtime_id: 2,
            direction: PaintingDirection::North
        };
        self.send_packet(reply)?;

        Ok(())
    }
}
