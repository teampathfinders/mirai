

use std::sync::Arc;

use proto::bedrock::{Animate, CommandOutput, CommandOutputMessage, CommandOutputType, CommandRequest, FormResponseData, ParsedCommand, RequestAbility, SettingsCommand, TextData, TextMessage, TickSync, UpdateSkin, PlayerAuthInput};

use util::Deserialize;

use super::BedrockUser;

impl BedrockUser {
    pub fn handle_settings_command(&self, packet: Vec<u8>) -> anyhow::Result<()> {
        let request = SettingsCommand::deserialize(packet.as_ref())?;
        dbg!(request);

        Ok(())
    }

    pub fn handle_tick_sync(&self, packet: Vec<u8>) -> anyhow::Result<()> {
        let _request = TickSync::deserialize(packet.as_ref())?;
        // TODO: Implement tick synchronisation
        Ok(())
        // let response = TickSync {
        //     request_tick: request.request_tick,
        //     response_tick: self.level.
        // };
        // self.send(response)
    }

    pub async fn handle_text_message(self: &Arc<Self>, packet: Vec<u8>) -> anyhow::Result<()> {
        let request = TextMessage::deserialize(packet.as_ref())?;
        if let TextData::Chat {
            ..
        } = request.data {
            // Check that the source is equal to the player name to prevent spoofing.
            #[cfg(not(debug_assertions))] // Allow modifications for development purposes.
            if self.name != source {
                self.kick("Illegal packet modifications detected")?;
                anyhow::bail!(
                    "Client attempted to spoof chat username. (actual: `{}`, spoofed: `{}`)",
                    actual, source
                );
            }

            // Send chat message to replication layer
            self.replicator.text_msg(&request).await?;

            // We must also return the packet to the client that sent it.
            // Otherwise their message won't be displayed in their own chat.
            self.broadcast(request)
        } else {
            // Only the server is allowed to create text raknet that are not of the chat type.
            anyhow::bail!("Client sent an illegally modified text message packet")
        }


    }

    pub fn handle_auth_input(&self, packet: Vec<u8>) -> anyhow::Result<()> {
        let input = PlayerAuthInput::deserialize(packet.as_ref())?;
        if input.input_data.0 != 0 {
            tracing::debug!("{:?}", input.input_data);
        }

        Ok(())
        // let move_player = MovePlayer {
        //     runtime_id: 1,
        //     mode: MovementMode::Normal,
        //     translation: input.position,
        //     pitch: input.pitch,
        //     yaw: input.yaw,
        //     head_yaw: input.head_yaw,
        //     on_ground: false,
        //     ridden_runtime_id: 0,
        //     teleport_cause: TeleportCause::Unknown,
        //     teleport_source_type: 0,
        //     tick: input.tick  
        // };
        // self.send(move_player)
    }

    pub fn handle_skin_update(&self, packet: Vec<u8>) -> anyhow::Result<()> {
        let request = UpdateSkin::deserialize(packet.as_ref())?;
        dbg!(&request);
        self.broadcast(request)
    }

    pub fn handle_ability_request(&self, packet: Vec<u8>) -> anyhow::Result<()> {
        let request = RequestAbility::deserialize(packet.as_ref())?;
        dbg!(request);

        Ok(())
    }

    pub fn handle_animation(&self, packet: Vec<u8>) -> anyhow::Result<()> {
        let request = Animate::deserialize(packet.as_ref())?;
        dbg!(request);

        Ok(())
    }

    pub fn handle_form_response(&self, packet: Vec<u8>) -> anyhow::Result<()> {
        let response = FormResponseData::deserialize(packet.as_ref())?;
        self.form_subscriber.handle_response(response)
    }

    pub fn handle_command_request(&self, packet: Vec<u8>) -> anyhow::Result<()> {
        let request = CommandRequest::deserialize(packet.as_ref())?;

        let command_list = self.level.get_commands();
        let result = ParsedCommand::parse(command_list, request.command);

        if let Ok(parsed) = result {
            let caller = self.xuid();
            let output = match parsed.name.as_str() {
                "gamerule" => {
                    self.level.on_gamerule_command(caller, parsed)
                },
                "effect" => {
                    self.level.on_effect_command(caller, parsed)
                }
                _ => todo!(),
            };

            if let Ok(message) = output {
                self.send(CommandOutput {
                    origin: request.origin,
                    request_id: request.request_id,
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
                    request_id: request.request_id,
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
                request_id: request.request_id,
                output_type: CommandOutputType::AllOutput,
                success_count: 0,
                output: &[CommandOutputMessage {
                    is_success: false,
                    message: &result.unwrap_err().to_string(),
                    parameters: &[],
                }],
            })?;
        }

        Ok(())
    }
}
