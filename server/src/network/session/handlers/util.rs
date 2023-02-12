use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use bytes::BytesMut;
use common::{BlockPosition, VResult, Vector3f, Vector3i};

use crate::network::{
    packets::{
        AddPainting, Animate, CameraShake, CameraShakeAction, CameraShakeType, ChangeDimension,
        CreditStatus, Difficulty, Dimension, GameMode, MessageType, MobEffectAction, MobEffectKind,
        MobEffectUpdate, NetworkChunkPublisherUpdate, PaintingDirection, PlaySound, RequestAbility,
        SetCommandsEnabled, SetDifficulty, SetPlayerGameMode, SetTime, SetTitle, ShowCredits,
        ShowProfile, SpawnExperienceOrb, TextMessage, TitleAction, ToastRequest, Transfer,
    },
    session::Session,
    Decodable,
};

impl Session {
    pub fn handle_text_message(&self, packet: BytesMut) -> VResult<()> {
        let request = TextMessage::decode(packet)?;
        tracing::info!("{request:?}");

        let transfer = Transfer {
            address: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 19132)),
        };
        self.send_packet(transfer)?;

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
