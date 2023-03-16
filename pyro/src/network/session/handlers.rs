




use util::{
    bail, Deserialize, Result,
};
use util::bytes::{MutableBuffer};

use crate::ParsedCommand;
use crate::{
    CommandOutput, CommandOutputMessage, CommandOutputType, CommandRequest,
    SettingsCommand,
};


use crate::{
    {
        Animate, MessageType, RequestAbility,
        TextMessage,
        UpdateSkin,
    },
    Session,
};

impl Session {
    pub fn handle_settings_command(&self, pk: MutableBuffer) -> Result<()> {
        let request = SettingsCommand::deserialize(pk.snapshot())?;
        tracing::info!("{request:?}");

        Ok(())
    }

    pub fn handle_text_message(&self, pk: MutableBuffer) -> Result<()> {
        let request = TextMessage::deserialize(pk.snapshot())?;
        if request.message_type != MessageType::Chat {
            bail!(Malformed, "Client is only allowed to send chat messages, received {:?} instead", request.message_type)
        }

        // We must also return the packet to the client that sent it.
        // Otherwise their message won't be displayed in their own chat.
        self.broadcast(request)
    }

    pub fn handle_skin_update(&self, pk: MutableBuffer) -> Result<()> {
        let request = UpdateSkin::deserialize(pk.snapshot())?;
        self.broadcast(request)
    }

    pub fn handle_ability_request(&self, pk: MutableBuffer) -> Result<()> {
        let request = RequestAbility::deserialize(pk.snapshot())?;
        tracing::info!("{request:?}");

        Ok(())
    }

    pub fn handle_animation(&self, pk: MutableBuffer) -> Result<()> {
        let _request = Animate::deserialize(pk.snapshot())?;

        Ok(())
    }

    pub fn handle_command_request(&self, pk: MutableBuffer) -> Result<()> {
        let request = CommandRequest::deserialize(pk.snapshot())?;

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
