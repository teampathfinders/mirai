

use std::sync::Arc;

use proto::bedrock::{Animate, CommandRequest, DisconnectReason, FormResponseData, PlayerAuthInput, RequestAbility, SettingsCommand, TextData, TextMessage, TickSync, UpdateSkin};

use util::Deserialize;

use super::BedrockUser;

impl BedrockUser {
    /// Handles a [`SettingsCommand`] packet used to adjust a world setting.
    pub fn handle_settings_command(&self, packet: Vec<u8>) -> anyhow::Result<()> {
        let request = SettingsCommand::deserialize(packet.as_ref())?;
        tracing::debug!("{request:?}");

        Ok(())
    }

    /// Handles a [`TickSync`] packet used to synchronise ticks between the client and server.
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

    /// Handles a [`TextMessage`] packet sent when a client wants to send a chat message.
    #[tracing::instrument(
        skip_all,
        name = "BedrockUser::handle_text_message"
        fields(
            username = %self.name().unwrap_or("<unknown>"),
            msg
        )
    )]
    pub async fn handle_text_message(self: &Arc<Self>, packet: Vec<u8>) -> anyhow::Result<()> {
        let request = TextMessage::deserialize(packet.as_ref())?;
        if let TextData::Chat {
            source, message
        } = request.data {
            tracing::Span::current().record("msg", message);

            let name = self.name()?;
            // Check that the source is equal to the player name to prevent spoofing.
            if name != source {
                tracing::warn!("Client and text message name do not match. Kicking them for forbidden modifications");
                return self.kick_with_reason("Illegal packet modifications detected", DisconnectReason::BadPacket).await
            }

            tracing::info!("<{source}>: {message}");

            // We must also return the packet to the client that sent it.
            // Otherwise their message won't be displayed in their own chat.
            self.broadcast(request)
        } else {
            // Only the server is allowed to create text raknet that are not of the chat type.
            tracing::warn!("Client sent an illegal message type. Kicking them for forbidden modifications");
            self.kick_with_reason("Illegal packet received", DisconnectReason::BadPacket).await
        }
    }

    /// Handles a [`PlayerAuthInput`] packet. These are sent every tick and are used
    /// for server authoritative player movement.
    pub fn handle_auth_input(&self, packet: Vec<u8>) -> anyhow::Result<()> {
        let input = PlayerAuthInput::deserialize(packet.as_ref())?;
        if input.input_data.0 != 0 {
            tracing::debug!("{:?}", input.input_data);
        }

        Ok(())
    }

    /// Handles an [`UpdateSkin`] packet.
    pub fn handle_skin_update(&self, packet: Vec<u8>) -> anyhow::Result<()> {
        let request = UpdateSkin::deserialize(packet.as_ref())?;
        tracing::debug!("{request:?}");
        self.broadcast(request)
    }

    /// Handles an [`AbilityRequest`] packet.
    pub fn handle_ability_request(&self, packet: Vec<u8>) -> anyhow::Result<()> {
        let request = RequestAbility::deserialize(packet.as_ref())?;
        tracing::debug!("{request:?}");
        
        Ok(())
    }

    /// Handles an [`Animation`] packet.
    pub fn handle_animation(&self, packet: Vec<u8>) -> anyhow::Result<()> {
        let request = Animate::deserialize(packet.as_ref())?;
        tracing::debug!("{request:?}");
        
        Ok(())
    }

    /// Handles a [`FormResponseData`] packet. This packet is forwarded to the forms [`Subscriber`](crate::forms::response::Subscriber)
    /// which will properly handle the response.
    /// 
    /// # Errors
    /// 
    /// May return an error if the packet fails to deserialize or handling a form response fails.
    pub fn handle_form_response(&self, packet: Vec<u8>) -> anyhow::Result<()> {
        let response = FormResponseData::deserialize(packet.as_ref())?;
        self.forms.handle_response(response)
    }

    /// Handles a [`CommandRequest`] packet.
    /// 
    /// # Errors
    /// 
    /// May return an error if the packet fails to deserialize or executing the command fails.
    pub async fn handle_command_request(&self, packet: Vec<u8>) -> anyhow::Result<()> {
        let request = CommandRequest::deserialize(packet.as_ref())?;
        let _callback = self.commands.request(request).await?;

        // let response = callback.await?;
        // dbg!(response);

        Ok(())
    }
}
