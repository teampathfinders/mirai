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
            effect_kind: MobEffectKind::HealthBoost,
            particles: true,
            amplifier: 255,
            operation: MobEffectOperation::Add,
            duration: u32::MAX,
        };
        self.send_packet(reply2)?;

        let reply3 = SetHealth { health: 10 };
        self.send_packet(reply3)?;

        // let reply = TextMessage {
        //     message_type: MessageType::System,
        //     needs_translation: false,
        //     source_name: "Server".to_owned(),
        //     message: "Popup".to_owned(),
        //     parameters: Vec::new(),
        //     xuid: request.xuid,
        //     platform_chat_id: request.platform_chat_id,
        // };
        // self.send_packet(reply)?;

        Ok(())
    }
}
