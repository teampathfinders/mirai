use std::num::NonZeroU64;

use bytes::{BufMut, BytesMut};

use crate::config::SERVER_CONFIG;
use crate::crypto::encrypt::perform_key_exchange;
use crate::error::VexResult;
use crate::network::packets::{ClientCacheStatus, GamePacket, Login, NetworkSettings, Packet, PacketBatch, PlayStatus, RequestNetworkSettings, ServerToClientHandshake, Status};
use crate::network::raknet::frame::{Frame, FrameBatch};
use crate::network::raknet::reliability::Reliability;
use crate::network::session::send_queue::SendPriority;
use crate::network::session::session::Session;
use crate::network::traits::{Decodable, Encodable};

impl Session {
    pub fn handle_client_cache_status(&self, mut task: BytesMut) -> VexResult<()> {
        let request = ClientCacheStatus::decode(task)?;
        tracing::debug!("{request:?} {self:?}");

        // Unused

        Ok(())
    }

    pub fn handle_login(&self, mut task: BytesMut) -> VexResult<()> {
        let request = Login::decode(task)?;
        tracing::info!("{request:?}");

        let data = perform_key_exchange(&request.identity.public_key)?;

        self.identity.set(request.identity)?;
        self.user_data.set(request.user_data)?;

        self.send_packet(ServerToClientHandshake {
            jwt: data.jwt.as_str()
        })?;

        Ok(())
    }

    pub fn handle_request_network_settings(&self, mut task: BytesMut) -> VexResult<()> {
        let request = RequestNetworkSettings::decode(task)?;
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
        tracing::trace!("Sent network settings");

        Ok(())
    }
}
