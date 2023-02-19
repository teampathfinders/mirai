use bytes::{Bytes, BytesMut};

use crate::network::packets::login::{OnlinePing, OnlinePong};
use crate::network::raknet::packets::ConnectionRequest;
use crate::network::raknet::packets::ConnectionRequestAccepted;
use crate::network::raknet::packets::NewIncomingConnection;
use crate::network::raknet::Frame;
use crate::network::raknet::Reliability;
use crate::network::session::send::PacketConfig;
use crate::network::session::send_queue::SendPriority;
use crate::network::session::session::Session;
use common::VResult;
use common::{Deserialize, Serialize};

impl Session {
    /// Handles a [`ConnectionRequest`] packet.
    pub fn handle_connection_request(&self, pk: Bytes) -> VResult<()> {
        let request = ConnectionRequest::deserialize(pk)?;
        let reply = ConnectionRequestAccepted {
            client_address: self.address,
            request_time: request.time,
        }
        .serialize()?;

        self.send_raw_buffer(reply);
        Ok(())
    }

    /// Handles a [`NewIncomingConnection`] packet.
    pub fn handle_new_incoming_connection(&self, pk: Bytes) -> VResult<()> {
        let request = NewIncomingConnection::deserialize(pk)?;
        Ok(())
    }

    /// Handles an [`OnlinePing`] packet.
    pub fn handle_online_ping(&self, pk: Bytes) -> VResult<()> {
        let ping = OnlinePing::deserialize(pk)?;
        let pong = OnlinePong {
            ping_time: ping.time,
            pong_time: ping.time,
        };

        let pong = pong.serialize()?;
        self.send_raw_buffer_with_config(
            pong,
            PacketConfig {
                reliability: Reliability::Unreliable,
                priority: SendPriority::Low,
            },
        );

        Ok(())
    }
}
