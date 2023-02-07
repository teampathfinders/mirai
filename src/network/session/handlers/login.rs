use std::num::NonZeroU64;
use std::sync::atomic::Ordering;

use anyhow::{bail, Context};
use bytes::{BufMut, BytesMut};
use jsonwebtoken::jwk::KeyOperations::Encrypt;

use crate::config::SERVER_CONFIG;
use crate::crypto::Encryptor;
use crate::network::packets::{ClientCacheStatus, ClientToServerHandshake, GamePacket, Login, NETWORK_VERSION, NetworkSettings, Packet, PlayStatus, RequestNetworkSettings, ResourcePackClientResponse, ResourcePacksInfo, ResourcePackStack, ServerToClientHandshake, Status};
use crate::network::raknet::{Frame, FrameBatch};
use crate::network::raknet::Reliability;
use crate::network::session::send_queue::SendPriority;
use crate::network::session::session::Session;
use crate::network::traits::{Decodable, Encodable};

impl Session {
    /// Handles a [`ClientCacheStatus`] packet.
    pub fn handle_client_cache_status(&self, mut packet: BytesMut) -> anyhow::Result<()> {
        let request = ClientCacheStatus::decode(packet)?;
        // Unused

        Ok(())
    }

    pub fn handle_resource_pack_client_response(&self, mut packet: BytesMut) -> anyhow::Result<()> {
        let request = ResourcePackClientResponse::decode(packet)?;
        tracing::info!("{request:?}");

        Ok(())
    }

    pub fn handle_client_to_server_handshake(&self, mut packet: BytesMut) -> anyhow::Result<()> {
        ClientToServerHandshake::decode(packet)?;
        tracing::trace!("Successfully initiated encryption");

        let response = PlayStatus {
            status: Status::LoginSuccess
        };
        self.send_packet(response)?;

        // TODO: Implement resource packs
        let pack_info = ResourcePacksInfo {
            required: false,
            scripting_enabled: false,
            forcing_server_packs: false,
            behavior_info: vec![],
            resource_info: vec![],
        };
        self.send_packet(pack_info)?;

        let pack_stack = ResourcePackStack {
            forced_to_accept: false,
            resource_packs: vec![],
            behavior_packs: vec![],
            game_version: "1.19".to_string(),
            experiments: vec![],
            experiments_previously_toggled: false,
        };
        self.send_packet(pack_stack)?;

        Ok(())
    }

    /// Handles a [`Login`] packet.
    pub async fn handle_login(&self, mut packet: BytesMut) -> anyhow::Result<()> {
        let request = Login::decode(packet)?;
        tracing::info!("{} has joined the server", request.identity.display_name);

        let (encryptor, jwt) = Encryptor::new(&request.identity.public_key)?;

        self.identity.set(request.identity)?;
        self.user_data.set(request.user_data)?;

        // Flush packets before enabling encryption
        self.flush().await?;

        self.send_packet(ServerToClientHandshake { jwt: jwt.as_str() })?;
        self.encryptor
            .set(encryptor)
            .context("Encryptor was already set")?;

        tracing::trace!("Initiating encryption...");

        Ok(())
    }

    /// Handles a [`RequestNetworkSettings`] packet.
    pub fn handle_request_network_settings(&self, mut packet: BytesMut) -> anyhow::Result<()> {
        let request = RequestNetworkSettings::decode(packet)?;
        if request.protocol_version != NETWORK_VERSION {
            if request.protocol_version > NETWORK_VERSION {
                let response = PlayStatus {
                    status: Status::FailedServer,
                };
                self.send_packet(response)?;

                bail!("Client is using a newer protocol, disconnecting them...");
            } else {
                let response = PlayStatus {
                    status: Status::FailedClient,
                };
                self.send_packet(response)?;

                bail!("Client is using an older protocol, disconnecting them...");
            }
        }

        // TODO: Disconnect client if incompatible protocol.

        let response = {
            let lock = SERVER_CONFIG.read();
            NetworkSettings {
                compression_algorithm: lock.compression_algorithm,
                compression_threshold: lock.compression_threshold,
                client_throttle: lock.client_throttle,
            }
        };

        self.send_packet(response)?;
        self.compression_enabled.store(true, Ordering::SeqCst);

        tracing::trace!("Sent network settings");

        Ok(())
    }
}
