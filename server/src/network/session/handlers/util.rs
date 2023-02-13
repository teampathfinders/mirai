use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use bytes::BytesMut;
use common::{BlockPosition, Decodable, VResult, Vector3f, Vector3i, Vector4f};

use crate::network::{
    packets::{
        AddPainting, Animate, CameraShake, CameraShakeAction, CameraShakeType,
        ChangeDimension, ClientBoundDebugRenderer, CommandRequest,
        CreditsStatus, CreditsUpdate, DebugRendererAction, Difficulty,
        GameMode, MessageType, MobEffectAction, MobEffectKind, MobEffectUpdate,
        NetworkChunkPublisherUpdate, PaintingDirection, PlaySound, PlayerFog,
        RequestAbility, SetCommandsEnabled, SetDifficulty, SetPlayerGameMode,
        SetTime, SetTitle, ShowProfile, SpawnExperienceOrb, TextMessage,
        TitleAction, ToastRequest, Transfer,
    },
    session::Session,
};

impl Session {
    pub fn handle_text_message(&self, packet: BytesMut) -> VResult<()> {
        let request = TextMessage::decode(packet)?;
        tracing::info!("{request:?}");

        let renderer = ClientBoundDebugRenderer {
            action: DebugRendererAction::AddCube,
            color: Vector4f::from([1.0, 1.0, 1.0, 1.0]),
            position: Vector3f::from([1.0, 1.0, 1.0]),
            text: "Hello, World!".to_owned(),
            duration: 10_000,
        };
        self.send_packet(renderer)?;

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
