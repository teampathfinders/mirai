use std::num::NonZeroU64;

use bytes::BytesMut;

use crate::config::SERVER_CONFIG;
use crate::error::VexResult;
use crate::network::packets::{ClientCacheStatus, Login, NetworkSettings, Packet, PacketBatch, RequestNetworkSettings, ServerToClientHandshake};
use crate::network::raknet::frame::Frame;
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

        self.identity.set(request.identity)?;
        self.user_data.set(request.user_data)?;

        // TODO: Send handshakes

        Ok(())
    }

    pub fn handle_request_network_settings(&self, mut task: BytesMut) -> VexResult<()> {
        let request = RequestNetworkSettings::decode(task)?;
        // TODO: Disconnect client if incompatible protocol.

        let mut reply = {
            let lock = SERVER_CONFIG.read();
            Packet::new(NetworkSettings {
                compression_algorithm: lock.compression_algorithm,
                compression_threshold: lock.compression_threshold,
                client_throttle: lock.client_throttle,
            })
                .subclients(0, 0)
        };

        let mut batch = PacketBatch::new().add(reply)?.encode()?;
        self.send_queue.insert(
            SendPriority::Medium,
            Frame::new(Reliability::ReliableOrdered, batch),
        );

        Ok(())
    }
}
