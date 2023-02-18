use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::time::Duration;

use bytes::BytesMut;
use common::{
    bail, BlockPosition, Deserialize, VResult, Vector3f, Vector3i, Vector4f,
};

use crate::command::ParsedCommand;
use crate::network::packets::command::{
    CommandOutput, CommandOutputMessage, CommandOutputType, CommandRequest,
    SettingsCommand,
};
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
        let request = SettingsCommand::deserialize(packet)?;
        tracing::info!("{request:?}");

        Ok(())
    }

    pub fn handle_text_message(&self, packet: BytesMut) -> VResult<()> {
        let request = TextMessage::deserialize(packet)?;
        if request.message_type != MessageType::Chat {
            bail!(BadPacket, "Client is only allowed to send chat messages, received {:?} instead", request.message_type)
        }

        // We must also return the packet to the client that sent it.
        // Otherwise their message won't be displayed in their own chat.
        self.broadcast(request)
    }

    pub fn handle_skin_update(&self, packet: BytesMut) -> VResult<()> {
        let request = UpdateSkin::deserialize(packet)?;
        self.broadcast(request)
    }

    pub fn handle_ability_request(&self, packet: BytesMut) -> VResult<()> {
        let request = RequestAbility::deserialize(packet)?;
        tracing::info!("{request:?}");

        Ok(())
    }

    pub fn handle_animation(&self, packet: BytesMut) -> VResult<()> {
        let request = Animate::deserialize(packet)?;
        tracing::info!("{request:?}");

        Ok(())
    }

    pub fn handle_command_request(&self, packet: BytesMut) -> VResult<()> {
        let request = CommandRequest::deserialize(packet)?;
        tracing::info!("{request:?}");

        let command_list = self.level_manager.get_commands();
        let result = ParsedCommand::parse(command_list, &request.command);

        if let Ok(parsed) = result {
            let output = match parsed.name.as_str() {
                "gamerule" => {
                    self.level_manager.handle_gamerule_command(parsed)
                }
                _ => todo!(),
            };

            if let Ok(message) = output {
                self.send(CommandOutput {
                    origin: request.origin,
                    request_id: &request.request_id,
                    output_type: CommandOutputType::AllOutput,
                    success_count: 1,
                    output: &[CommandOutputMessage {
                        is_success: true,
                        message: &message,
                        parameters: &[],
                    }],
                })?;
            } else {
                self.send(CommandOutput {
                    origin: request.origin,
                    request_id: &request.request_id,
                    output_type: CommandOutputType::AllOutput,
                    success_count: 0,
                    output: &[CommandOutputMessage {
                        is_success: false,
                        message: &output.unwrap_err().to_string(),
                        parameters: &[],
                    }],
                })?;
            }
        } else {
            self.send(CommandOutput {
                origin: request.origin,
                request_id: &request.request_id,
                output_type: CommandOutputType::AllOutput,
                success_count: 0,
                output: &[CommandOutputMessage {
                    is_success: false,
                    message: &result.unwrap_err(),
                    parameters: &[],
                }],
            })?;
        }

        Ok(())
    }
}
