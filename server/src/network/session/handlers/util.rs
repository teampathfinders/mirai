use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::time::Duration;

use bytes::BytesMut;
use common::{
    bail, BlockPosition, Decodable, VResult, Vector3f, Vector3i, Vector4f,
};

use crate::command::ParsedCommand;
use crate::network::packets::command::{CommandRequest, SettingsCommand};
use crate::network::packets::login::{ItemStack, ItemType, PermissionLevel};
use crate::network::packets::{AbilityData, AddPlayer};
use crate::network::{
    packets::{
        AddPainting, Animate, CameraShake, CameraShakeAction, CameraShakeType,
        ChangeDimension, ClientBoundDebugRenderer, CreditsStatus,
        CreditsUpdate, DebugRendererAction, Difficulty, GameMode, GameRule,
        GameRulesChanged, MessageType, MobEffectAction, MobEffectKind,
        MobEffectUpdate, NetworkChunkPublisherUpdate, PaintingDirection,
        PlaySound, RequestAbility, SetCommandsEnabled, SetDifficulty,
        SetPlayerGameMode, SetTime, SetTitle, ShowProfile, SpawnExperienceOrb,
        TextMessage, TitleAction, ToastRequest, Transfer, UpdateFogStack,
        UpdateSkin,
    },
    session::Session,
};

impl Session {
    pub fn handle_settings_command(&self, packet: BytesMut) -> VResult<()> {
        let request = SettingsCommand::decode(packet)?;
        tracing::info!("{request:?}");

        Ok(())
    }

    pub fn handle_text_message(&self, packet: BytesMut) -> VResult<()> {
        let request = TextMessage::decode(packet)?;
        if request.message_type != MessageType::Chat {
            bail!(BadPacket, "Client is only allowed to send chat messages, received {:?} instead", request.message_type)
        }

        // self.broadcast_others(AddPlayer {
        //     uuid: *self.get_uuid()?,
        //     username: self.get_display_name()?,
        //     runtime_id: self.runtime_id,
        //     position: Vector3f::from([0.0, 0.0, 0.0]),
        //     velocity: Vector3f::from([0.0, 0.0, 0.0]),
        //     rotation: Vector3f::from([0.0, 0.0, 0.0]),
        //     game_mode: GameMode::Creative,
        //     held_item: ItemStack {
        //         item_type: ItemType {
        //             network_id: 0,
        //             metadata: 0
        //         },
        //         runtime_id: 0,
        //         count: 0,
        //         nbt_data: nbt::Value::End,
        //         can_be_placed_on: vec![],
        //         can_break: vec![],
        //         has_network_id: false,
        //     },
        //     metadata: HashMap::new(),
        //     ability_data: AbilityData {
        //         entity_id: 2,
        //         permission_level: PermissionLevel::Operator,
        //         command_permission_level: CommandPermissionLevel::Admin,
        //         layers: &[],
        //     },
        //     links: &[],
        //     device_id: &self.get_user_data()?.device_id,
        //     device_os: self.get_device_os()?,
        // })?;

        // We must also return the packet to the client that sent it.
        // Otherwise their message won't be displayed in their own chat.
        self.broadcast(request)
    }

    pub fn handle_skin_update(&self, packet: BytesMut) -> VResult<()> {
        let request = UpdateSkin::decode(packet)?;
        self.broadcast(request)
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

        let command_list = self.level_manager.get_commands();
        let parsed = ParsedCommand::parse(command_list, &request.command).unwrap();

        tracing::info!("{parsed:?}");

        Ok(())
    }
}
