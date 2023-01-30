use std::num::NonZeroU64;
use std::sync::atomic::Ordering;

use anyhow::{bail, Context};
use bytes::{BufMut, BytesMut};
use jsonwebtoken::jwk::KeyOperations::Encrypt;

use crate::config::SERVER_CONFIG;
use crate::crypto::Encryptor;
use crate::network::packets::{ClientCacheStatus, ClientToServerHandshake, GamePacket, Login, NETWORK_VERSION, NetworkSettings, Packet, PlayStatus, RequestNetworkSettings, ServerToClientHandshake, Status};
use crate::network::raknet::{Frame, FrameBatch};
use crate::network::raknet::Reliability;
use crate::network::session::send_queue::SendPriority;
use crate::network::session::session::Session;
use crate::network::traits::{Decodable, Encodable};

impl Session {
    /// Handles a [`ClientCacheStatus`] packet.
    pub fn handle_client_cache_status(&self, mut task: BytesMut) -> anyhow::Result<()> {
        let request = ClientCacheStatus::decode(task)?;
        tracing::debug!("{request:?} {self:?}");

        // Unused

        Ok(())
    }

    pub fn handle_client_to_server_handshake(&self, mut packet: BytesMut) -> anyhow::Result<()> {
        ClientToServerHandshake::decode(packet)?;
        tracing::trace!("Successfully initiated encryption");

        let response = PlayStatus {
            status: Status::LoginSuccess
        };
        self.send_packet(response)?;

        Ok(())
    }

    /// Handles a [`Login`] packet.
    pub async fn handle_login(&self, mut task: BytesMut) -> anyhow::Result<()> {
        let request = Login::decode(task)?;
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
    pub fn handle_request_network_settings(&self, mut task: BytesMut) -> anyhow::Result<()> {
        let request = RequestNetworkSettings::decode(task)?;
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
