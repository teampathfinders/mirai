use bytes::BytesMut;

use crate::error::VexResult;
use crate::network::packets::online_ping::OnlinePing;
use crate::network::packets::online_pong::OnlinePong;
use crate::network::raknet::frame::Frame;
use crate::network::raknet::packets::connection_request::ConnectionRequest;
use crate::network::raknet::packets::connection_request_accepted::ConnectionRequestAccepted;
use crate::network::raknet::packets::new_incoming_connection::NewIncomingConnection;
use crate::network::raknet::reliability::Reliability;
use crate::network::session::leaving::PacketConfig;
use crate::network::session::send_queue::SendPriority;
use crate::network::session::session::Session;
use crate::network::traits::{Decodable, Encodable};

impl Session {
    pub fn handle_connection_request(&self, task: BytesMut) -> VexResult<()> {
        let request = ConnectionRequest::decode(task)?;
        let response = ConnectionRequestAccepted {
            client_address: self.address,
            request_time: request.time,
        }.encode()?;

        self.send_raw_buffer(response);
        tracing::trace!("Accepted connection request");

        Ok(())
    }

    pub fn handle_new_incoming_connection(&self, task: BytesMut) -> VexResult<()> {
        let request = NewIncomingConnection::decode(task)?;
        Ok(())
    }

    pub fn handle_online_ping(&self, task: BytesMut) -> VexResult<()> {
        let ping = OnlinePing::decode(task)?;
        let pong = OnlinePong {
            ping_time: ping.time,
            pong_time: ping.time,
        };

        let pong = pong.encode()?;
        self.send_raw_buffer_with_config(pong, PacketConfig {
            reliability: Reliability::Unreliable,
            priority: SendPriority::Low,
        });

        Ok(())
    }
}
