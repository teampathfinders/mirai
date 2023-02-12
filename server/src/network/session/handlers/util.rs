use bytes::BytesMut;
use common::{BlockPosition, VResult, Vector3f, Vector3i};

use crate::network::{
    packets::{
        AddPainting, ChangeDimension, CreditStatus, Difficulty, Dimension, GameMode, MessageType,
        MobEffectAction, MobEffectKind, MobEffectUpdate, NetworkChunkPublisherUpdate,
        PaintingDirection, PlaySound, SetCommandsEnabled, SetDifficulty, SetPlayerGameMode,
        SetTime, SetTitle, ShowCredits, ShowProfile, TextMessage, TitleAction, ToastRequest, SpawnExperienceOrb, RequestAbility, Animate,
    },
    session::Session,
    Decodable,
};

impl Session {
    pub fn handle_text_message(&self, packet: BytesMut) -> VResult<()> {
        let request = TextMessage::decode(packet)?;
        tracing::info!("{request:?}");

        let reply = SetTitle {
            remain_duration: 40,
            xuid: self.get_xuid()?.to_string(),
            action: TitleAction::SetActionBar,
            text: format!("You said {}", request.message),
            platform_online_id: "".to_owned(),
            fade_in_duration: 10,
            fade_out_duration: 10,
        };
        self.send_packet(reply)?;

        let reply2 = ToastRequest {
            title: request.message.clone(),
            message: "Do not move".to_owned()
        };
        self.send_packet(reply2)?;

        let reply3 = SpawnExperienceOrb {
            position: Vector3f::from([0.0, 0.0, -2.0]),
            amount: 1000
        };
        self.send_packet(reply3)?;

        let reply4 = AddPainting {
            position: Vector3f::from([0.0, 0.0, 10.0]),
            direction: PaintingDirection::North,
            name: "BurningSkull".to_owned(),
            runtime_id: 2
        };
        self.send_packet(reply4)?;

        Ok(())
    }

    pub fn handle_ability_request(&self, packet: BytesMut) -> VResult<()> {
        let request = RequestAbility::decode(packet)?;
        tracing::info!("{request:?}");

        Ok(())
    }

    pub fn handle_animation(&self, packet: BytesMut) -> VResult<()> {
        let request = Animate::decode(packet)?;
        tracing::info!("{request:?}");

        Ok(())
    }
}
