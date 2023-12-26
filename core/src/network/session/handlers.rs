use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::str::FromStr;
use level::Dimension;
use util::{bail, Deserialize, Result, TryExpect, Vector};
use util::bytes::MutableBuffer;

use crate::network::{CommandOutput, CommandOutputMessage, CommandOutputType, CommandRequest, FormRequest, FormResponse, SettingsCommand, SubChunkResponse, TextData, TickSync, Transfer};
use crate::network::{
    {
        Animate, RequestAbility,
        TextMessage,
        UpdateSkin,
    },
    Session,
};
use crate::command::ParsedCommand;
use crate::form::{FormButton, FormElement, FormInput, FormLabel, FormSlider, Form, Modal, FormButtonImage, FormDropdown, FormToggle, FormStepSlider, CustomForm};

impl Session {
    pub fn process_settings_command(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        let request = SettingsCommand::deserialize(packet.snapshot())?;
        tracing::info!("{request:?}");

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
            if actual != source {
                self.kick("Illegal packet modifications detected")?;
                anyhow::bail!(
                    "Client attempted to spoof chat username. (actual: `{}`, spoofed: `{}`)",
                    actual, source
                );
            }

            // We must also return the packet to the client that sent it.
            // Otherwise their message won't be displayed in their own chat.
            self.broadcast(request)
        } else {
            // Only the server is allowed to create text packets that are not of the chat type.
            anyhow::bail!("Client sent an illegally modified text message packet")
        }


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

    pub fn process_form_response(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        let response = FormResponse::deserialize(packet.snapshot())?;
        let raw: (&str, &str) = serde_json::from_str(response.response_data.unwrap()).unwrap();
            
        dbg!(raw);
        // self.send(Transfer {
        //     addr: raw.0, port: raw.1.parse().unwrap()
        // })?;

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
                    let out = self.level.on_effect_command(caller, parsed)?;

                    // let custom_form = serde_json::to_string(&CustomForm {
                    //     title: &String::from_utf8_lossy(&[0xee, 0x84, 0x88, 0x20]),
                    //     content: &[
                    //         FormElement::Slider(FormSlider {
                    //             label: "Example slider",
                    //             default: 0.0,
                    //             max: 1.0,
                    //             min: 0.0,
                    //             step: 0.1
                    //         }),
                    //         FormElement::Dropdown(FormDropdown {
                    //             label: "Example dropdown",
                    //             default: 0,
                    //             options: &[
                    //                 "Option 1",
                    //                 "Option 2",
                    //                 "Option 3",
                    //                 "Option 4"
                    //             ]
                    //         }),
                    //         FormElement::Toggle(FormToggle {
                    //             label: "Example toggle",
                    //             default: false
                    //         }),
                    //         FormElement::Input(FormInput {
                    //             label: "Example input",
                    //             placeholder: "Example placeholder",
                    //             default: "Default input"
                    //         }),
                    //         FormElement::StepSlider(FormStepSlider {
                    //             label: "Example step slider",
                    //             steps: &[
                    //                 "Step 1",
                    //                 "Step 2",
                    //                 "Step 3",
                    //                 "Step 4"
                    //             ],
                    //             default: 0
                    //         })
                    //     ]
                    // })?;

                    let custom_form = serde_json::to_string(&CustomForm {
                        title: "Transfer to other server",
                        content: &[
                            FormElement::Input(FormInput {
                                label: "Address",
                                placeholder: "",
                                default: ""
                            }),
                            FormElement::Input(FormInput {
                                label: "Port",
                                placeholder: "",
                                default: "19132"
                            })
                        ]
                    }).unwrap();

                    let modal_request = FormRequest {
                        id: 0,
                        data: &custom_form
                    };
                    self.send(modal_request);
                    Ok(out)
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
