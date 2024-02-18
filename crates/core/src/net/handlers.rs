

use std::sync::Arc;

use futures::StreamExt;
use proto::bedrock::{Animate, CommandOutput, CommandOutputMessage, CommandOutputType, CommandRequest, DisconnectReason, FormResponseData, HudElement, HudVisibility, PlayerAuthInput, RequestAbility, SetHud, SettingsCommand, TextData, TextMessage, TickSync, UpdateSkin};

use util::{RVec, Deserialize, CowSlice, Vector};

use crate::level::RegionQuery;

use super::BedrockClient;

impl BedrockClient {
    /// Handles a [`SettingsCommand`] packet used to adjust a world setting.
    pub fn handle_settings_command(&self, packet: RVec) -> anyhow::Result<()> {
        let request = SettingsCommand::deserialize(packet.as_ref())?;
        tracing::debug!("{request:?}");

        Ok(())
    }

    /// Handles a [`TickSync`] packet used to synchronise ticks between the client and server.
    pub fn handle_tick_sync(&self, packet: RVec) -> anyhow::Result<()> {
        let _request = TickSync::deserialize(packet.as_ref())?;
        // TODO: Implement tick synchronisation
        Ok(())
        // let response = TickSync {
        //     request_tick: request.request_tick,
        //     response_tick: self.level.
        // };
        // self.send(response)
    }

    /// Handles a [`TextMessage`] packet sent when a client wants to send a chat message.
    #[tracing::instrument(
        skip_all,
        name = "BedrockUser::handle_text_message"
        fields(
            username = %self.name().unwrap_or("<unknown>"),
            msg
        )
    )]
    pub fn handle_text_message(self: &Arc<Self>, packet: RVec) -> anyhow::Result<()> {
        let this = Arc::clone(self);
        tokio::spawn(async move {
            let request = RegionQuery::from_bounds([10, -4, 0], [0, 15, 10]);

            let instant = std::time::Instant::now();
            let mut stream = this.level.request_region(request);

            while let Some(chunk) = stream.next().await {
                if !chunk.data.is_empty() {
                    let coord = <Vector<i32, 3>>::from(chunk.index);
                    println!("{coord:?} {:?}", chunk.index);
                }          
            }
            println!("loaded chunks in {:?}", instant.elapsed());

            tracing::debug!("All chunks loaded");
        });

        let request = TextMessage::deserialize(packet.as_ref())?;
        if let TextData::Chat {
            source, message
        } = request.data {
            tracing::Span::current().record("msg", message);

            let name = self.name()?;
            // Check that the source is equal to the player name to prevent spoofing.
            if name != source {
                tracing::warn!("Client and text message name do not match. Kicking them for forbidden modifications");
                return self.kick_with_reason("Illegal packet modifications detected", DisconnectReason::BadPacket)
            }

            // We must also return the packet to the client that sent it.
            // Otherwise their message won't be displayed in their own chat.
            self.broadcast(request)
        } else {
            // Only the server is allowed to create text raknet that are not of the chat type.
            tracing::warn!("Client sent an illegal message type. Kicking them for forbidden modifications");
            self.kick_with_reason("Illegal packet received", DisconnectReason::BadPacket)
        }
    }

    /// Handles a [`PlayerAuthInput`] packet. These are sent every tick and are used
    /// for server authoritative player movement.
    pub fn handle_auth_input(&self, packet: RVec) -> anyhow::Result<()> {
        let input = PlayerAuthInput::deserialize(packet.as_ref())?;
        if input.input_data.0 != 0 {
            tracing::debug!("{:?}", input.input_data);
        }

        Ok(())
    }

    /// Handles an [`UpdateSkin`] packet.
    pub fn handle_skin_update(&self, packet: RVec) -> anyhow::Result<()> {
        let request = UpdateSkin::deserialize(packet.as_ref())?;
        tracing::debug!("{request:?}");
        self.broadcast(request)
    }

    /// Handles an [`AbilityRequest`] packet.
    pub fn handle_ability_request(&self, packet: RVec) -> anyhow::Result<()> {
        let request = RequestAbility::deserialize(packet.as_ref())?;
        tracing::debug!("{request:?}");
        
        Ok(())
    }

    /// Handles an [`Animation`] packet.
    pub fn handle_animation(&self, packet: RVec) -> anyhow::Result<()> {
        let request = Animate::deserialize(packet.as_ref())?;

        self.send(SetHud {
            elements: &[HudElement::Hotbar],
            visibibility: HudVisibility::Hide
        })?;

        tracing::debug!("{request:?}");
        
        Ok(())
    }

    /// Handles a [`FormResponseData`] packet. This packet is forwarded to the forms [`Subscriber`](crate::forms::response::Subscriber)
    /// which will properly handle the response.
    /// 
    /// # Errors
    /// 
    /// May return an error if the packet fails to deserialize or handling a form response fails.
    pub fn handle_form_response(&self, packet: RVec) -> anyhow::Result<()> {
        let response = FormResponseData::deserialize(packet.as_ref())?;
        self.forms.handle_response(response)
    }

    /// Handles a [`CommandRequest`] packet.
    /// 
    /// # Errors
    /// 
    /// May return an error if the packet fails to deserialize or executing the command fails.
    #[tracing::instrument(
        skip_all,
        name = "handle_command_request",
        fields(
            command,
            username = self.name().unwrap_or("<unknown>")
        )
    )]
    pub fn handle_command_request(self: Arc<Self>, packet: RVec) {
        // Command execution could take several ticks, await the result in a separate task
        // to avoid blocking the request handler.
        tokio::spawn(async move {
            let request = match CommandRequest::deserialize(packet.as_ref()) {
                Ok(req) => req,
                Err(err) => {
                    tracing::error!("Failed to deserialize `CommandRequest`: {err:#}");
                    return;
                }
            };
            tracing::Span::current().record("command", request.command);

            let receiver = match self.commands.execute(
                Arc::clone(&self),
                request.command.to_owned()
            ).await {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!("{e:#}"); 
                    return
                }
            };

            receiver.await.map_or_else(|_| tracing::error!("Command service shut down while awaiting execution"), |result| {
                let is_success = result.is_ok();
                let data = match result {
                    Ok(r) => r,
                    Err(r) => r
                };

                let messages = vec![CommandOutputMessage {
                    is_success,
                    message: data.message,
                    parameters: CowSlice::Owned(data.parameters)
                }];

                let output = CommandOutput {
                    success_count: if is_success { 1 } else { 0 },
                    request_id: request.request_id,
                    origin: request.origin,
                    output_type: CommandOutputType::AllOutput,
                    output: CowSlice::Owned(messages)
                }; 

                if let Err(err) = self.send(output) {
                    tracing::error!("Failed to send command output to client: {err:#}");
                }
            });
        });
    }
}
