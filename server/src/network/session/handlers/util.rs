use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use bytes::BytesMut;
use common::{BlockPosition, Decodable, VResult, Vector3f, Vector3i};

use crate::network::{
    packets::{
        AddPainting, Animate, CameraShake, CameraShakeAction, CameraShakeType,
        ChangeDimension, CommandRequest, CreditsStatus, CreditsUpdate,
        Difficulty, GameMode, MessageType, MobEffectAction, MobEffectKind,
        MobEffectUpdate, NetworkChunkPublisherUpdate, PaintingDirection,
        PlaySound, RequestAbility, SetCommandsEnabled, SetDifficulty,
        SetPlayerGameMode, SetTime, SetTitle, ShowProfile, SpawnExperienceOrb,
        TextMessage, TitleAction, ToastRequest, Transfer, PlayerFog,
    },
    session::Session,
};

impl Session {
    pub fn handle_text_message(&self, packet: BytesMut) -> VResult<()> {
        let request = TextMessage::decode(packet)?;
        tracing::info!("{request:?}");

        let fog_stack = PlayerFog {
            stack: vec!["minecraft:fog_ocean".to_owned()]
        };
        self.send_packet(fog_stack)?;

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
