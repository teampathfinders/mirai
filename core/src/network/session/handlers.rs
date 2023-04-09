use util::{
    bail, Deserialize, Result,
};
use util::bytes::MutableBuffer;

use crate::network::{Attribute, CommandOutput, CommandOutputMessage, CommandOutputType, CommandRequest, SettingsCommand, TextData, UpdateAttributes};
use crate::network::{
    {
        Animate, RequestAbility,
        TextMessage,
        UpdateSkin,
    },
    Session,
};
use crate::command::ParsedCommand;

impl Session {
    pub fn process_settings_command(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        let request = SettingsCommand::deserialize(packet.snapshot())?;
        tracing::info!("{request:?}");

        Ok(())
    }

    pub fn process_text_message(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        let request = TextMessage::deserialize(packet.snapshot())?;
        if !matches!(request.data, TextData::Chat { .. }) {
            anyhow::bail!("Client is only allowed to send chat messages");
        }

        // We must also return the packet to the client that sent it.
        // Otherwise their message won't be displayed in their own chat.
        self.broadcast(request)
    }

    pub fn process_skin_update(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        let request = UpdateSkin::deserialize(packet.snapshot())?;
        self.broadcast(request)
    }

    pub fn process_ability_request(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        let request = RequestAbility::deserialize(packet.snapshot())?;
        tracing::info!("{request:?}");

        Ok(())
    }

    pub fn process_animation(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        let _request = Animate::deserialize(packet.snapshot())?;

        Ok(())
    }

    pub fn process_command_request(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        let request = CommandRequest::deserialize(packet.snapshot())?;

        let command_list = self.level_manager.get_commands();
        let result = ParsedCommand::parse(command_list, request.command);

        if let Ok(parsed) = result {
            let output = match parsed.name.as_str() {
                "gamerule" => {
                    self.level_manager.execute_game_rule_command(parsed)
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
                    message: &result.unwrap_err(),
                    parameters: &[],
                }],
            })?;
        }

        Ok(())
    }
}
