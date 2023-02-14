use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use bytes::BytesMut;
use common::{BlockPosition, Decodable, VResult, Vector3f, Vector3i, Vector4f};

use crate::network::{
    packets::{
        AddPainting, Animate, CameraShake, CameraShakeAction, CameraShakeType,
        ChangeDimension, ClientBoundDebugRenderer, CommandRequest,
        CreditsStatus, CreditsUpdate, DebugRendererAction, Difficulty,
        GameMode, GameRule, GameRulesChanged, MessageType, MobEffectAction,
        MobEffectKind, MobEffectUpdate, NetworkChunkPublisherUpdate,
        PaintingDirection, PlaySound, RequestAbility, SetCommandsEnabled,
        SetDifficulty, SetPlayerGameMode, SetTime, SetTitle, ShowProfile,
        SpawnExperienceOrb, TextMessage, TitleAction, ToastRequest, Transfer,
        UpdateFogStack,
    },
    session::Session,
};

impl Session {
    pub fn handle_text_message(&self, packet: BytesMut) -> VResult<()> {
        let request = TextMessage::decode(packet)?;
        tracing::info!("{request:?}");

        let game_rules = GameRulesChanged {
            game_rules: &[GameRule::ShowCoordinates(false)],
        };
        self.send_packet(game_rules)?;

        let toast = ToastRequest {
            title: "Game Rule Updated",
            message: "Disabled the showcoordinates gamerule",
        };
        self.send_packet(toast)?;

        let fog = UpdateFogStack {
            stack: &["minecraft:fog_hell".to_owned()],
        };
        self.send_packet(fog)?;

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

    pub fn handle_command_request(&self, packet: BytesMut) -> VResult<()> {
        let request = CommandRequest::decode(packet)?;
        tracing::info!("{request:?}");

        if request.command == "/credits" {
            let credits = CreditsUpdate {
                runtime_id: 1,
                status: CreditsStatus::Start,
            };
            self.send_packet(credits)?;
        }

        Ok(())
    }
}
