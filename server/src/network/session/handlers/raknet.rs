use bytes::BytesMut;

use common::VResult;
use crate::network::packets::OnlinePing;
use crate::network::packets::OnlinePong;
use crate::network::raknet::Frame;
use crate::network::raknet::packets::ConnectionRequest;
use crate::network::raknet::packets::ConnectionRequestAccepted;
use crate::network::raknet::packets::NewIncomingConnection;
use crate::network::raknet::Reliability;
use crate::network::session::send::PacketConfig;
use crate::network::session::send_queue::SendPriority;
use crate::network::session::session::Session;
use crate::network::traits::{Decodable, Encodable};

impl Session {
    /// Handles a [`ConnectionRequest`] packet.
    pub fn handle_connection_request(&self, task: BytesMut) -> VResult<()> {
        let request = ConnectionRequest::decode(task)?;
        let response = ConnectionRequestAccepted {
            client_address: self.address,
            request_time: request.time,
        }
            .encode()?;

        self.send_raw_buffer(response);
        tracing::trace!("Accepted connection request");

        Ok(())
    }

    /// Handles a [`NewIncomingConnection`] packet.
    pub fn handle_new_incoming_connection(&self, task: BytesMut) -> VResult<()> {
        let request = NewIncomingConnection::decode(task)?;
        Ok(())
    }

    /// Handles an [`OnlinePing`] packet.
    pub fn handle_online_ping(&self, task: BytesMut) -> VResult<()> {
        let ping = OnlinePing::decode(task)?;
        let pong = OnlinePong {
            ping_time: ping.time,
            pong_time: ping.time,
        };

        let pong = pong.encode()?;
        self.send_raw_buffer_with_config(
            pong,
            PacketConfig {
                reliability: Reliability::Reliable,
                priority: SendPriority::Low,
            },
        );

        Ok(())
    }
}