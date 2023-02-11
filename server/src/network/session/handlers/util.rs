use bytes::BytesMut;
use common::{VResult, Vector3i};

use crate::network::{
    packets::{
        GameMode, MessageType, MobEffectKind, MobEffectOperation, MobEffectUpdate, PlaySound,
        SetHealth, SetPlayerGameMode, SetTime, ShowProfile, TextMessage,
    },
    session::Session,
    Decodable,
};

impl Session {
    pub fn handle_text_message(&self, packet: BytesMut) -> VResult<()> {
        let request = TextMessage::decode(packet)?;
        tracing::info!("{request:?}");

        let reply1 = SetPlayerGameMode {
            game_mode: GameMode::Survival,
        };
        self.send_packet(reply1)?;

        let reply2 = MobEffectUpdate {
            runtime_id: 1,
            effect_kind: MobEffectKind::FatalPoison,
            particles: true,
            amplifier: 255,
            operation: MobEffectOperation::Add,
            duration: i32::MAX,
        };
        self.send_packet(reply2)?;

        let reply3 = SetHealth { health: 10 };
        self.send_packet(reply3)?;

        Ok(())
    }
}
