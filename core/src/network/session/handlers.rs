use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::str::FromStr;
use proto::bedrock::{Animate, CommandOutput, CommandOutputMessage, CommandOutputType, CommandRequest, FormResponse, ParsedCommand, RequestAbility, SettingsCommand, TextData, TextMessage, TickSync, UpdateSkin};
use proto::types::Dimension;
use util::{bail, Deserialize, Result, TryExpect, Vector};
use util::MutableBuffer;

use crate::network::Session;
use crate::forms::{FormButton, FormElement, FormInput, FormLabel, FormSlider, MenuForm, Modal, FormButtonImage, FormDropdown, FormToggle, FormStepSlider, CustomForm};

impl Session {
    pub fn process_settings_command(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        let request = SettingsCommand::deserialize(packet.snapshot())?;
        dbg!(request);

        Ok(())
    }

    pub fn process_tick_sync(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        let request = TickSync::deserialize(packet.snapshot())?;
        // TODO: Implement tick synchronisation
        Ok(())
        // let response = TickSync {
        //     request_tick: request.request_tick,
        //     response_tick: self.level.
        // };
        // self.send(response)
    }

    pub fn process_text_message(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        let request = TextMessage::deserialize(packet.snapshot())?;
        if let TextData::Chat {
            source, ..
        } = request.data {
            let actual = &self.identity.get()
                .try_expect("Client does not have associated user data")?
                .display_name;

            // Check that the source is equal to the player name to prevent spoofing.
            #[cfg(not(debug_assertions))] // Allow modifications for development purposes.
            if actual != source {
                self.kick("Illegal packet modifications detected")?;
                anyhow::bail!(
                    "Client attempted to spoof chat username. (actual: `{}`, spoofed: `{}`)",
                    actual, source
                );
            }

            // Send chat message to replication layer
            self.replicator.pub_text_message(&request);

            // We must also return the packet to the client that sent it.
            // Otherwise their message won't be displayed in their own chat.
            self.broadcast(request)
        } else {
            // Only the server is allowed to create text raknet that are not of the chat type.
            anyhow::bail!("Client sent an illegally modified text message packet")
        }


    }

    pub fn process_skin_update(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        let request = UpdateSkin::deserialize(packet.snapshot())?;
        dbg!(&request);
        self.broadcast(request)
    }

    pub fn process_ability_request(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        let request = RequestAbility::deserialize(packet.snapshot())?;
        dbg!(request);

        Ok(())
    }

    pub fn process_animation(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        let request = Animate::deserialize(packet.snapshot())?;
        dbg!(request);

        Ok(())
    }

    pub fn process_form_response(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        let response = FormResponse::deserialize(packet.snapshot())?;
        dbg!(response);

        Ok(())
    }

    pub fn process_command_request(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        let request = CommandRequest::deserialize(packet.snapshot())?;

        let command_list = self.level.get_commands();
        let result = ParsedCommand::parse(command_list, request.command);

        if let Ok(parsed) = result {
            let caller = self.identity.get().unwrap().xuid;
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
                    message: &result.unwrap_err(),
                    parameters: &[],
                }],
            })?;
        }

        Ok(())
    }
}
