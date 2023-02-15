use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use bytes::BytesMut;
use common::{BlockPosition, Decodable, VResult, Vector3f, Vector3i, Vector4f, bail};

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
        UpdateFogStack, UpdateSkin,
    },
    session::Session,
};

impl Session {
    pub fn handle_text_message(&self, packet: BytesMut) -> VResult<()> {
        let request = TextMessage::decode(packet)?;
        if request.message_type != MessageType::Chat {
            bail!(BadPacket, "Client is only allowed to send chat messages, received {:?} instead", request.message_type)
        }

        // We must also return the packet to the client that sent it.
        // Otherwise their message won't be displayed in their own chat.
        self.broadcast(request)
    }

    pub fn handle_skin_update(&self, packet: BytesMut) -> VResult<()> {
        let request = UpdateSkin::decode(packet)?;
        tracing::debug!("{request:?}");

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
            self.send(credits)?;
        }

        Ok(())
    }
}
