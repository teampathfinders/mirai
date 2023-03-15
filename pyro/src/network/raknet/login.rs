

use crate::network::raknet::packets::ConnectionRequest;
use crate::network::raknet::packets::ConnectionRequestAccepted;
use crate::network::raknet::packets::NewIncomingConnection;
use crate::network::raknet::Frame;
use crate::network::raknet::Reliability;
use crate::network::session::Session;
use util::Result;
use util::{Deserialize, Serialize};
use util::bytes::{MutableBuffer, SharedBuffer};

use super::packets::ConnectedPing;
use super::packets::ConnectedPong;
use super::{PacketConfig, SendPriority};

impl Session {
    /// Handles a [`ConnectionRequest`] packet.
    pub fn handle_connection_request(&self, mut pk: MutableBuffer) -> Result<()> {
        let request = ConnectionRequest::deserialize(pk.snapshot())?;
        let reply = ConnectionRequestAccepted {
            client_address: self.raknet.address,
            request_time: request.time,
        };

        pk.clear();
        pk.reserve_to(reply.serialized_size());
        reply.serialize(&mut pk);

        self.send_raw_buffer(pk);
        Ok(())
    }

    /// Handles a [`NewIncomingConnection`] packet.
    pub fn handle_new_incoming_connection(&self, pk: MutableBuffer) -> Result<()> {
        let request = NewIncomingConnection::deserialize(pk.snapshot())?;
        Ok(())
    }

    /// Handles an [`OnlinePing`] packet.
    pub fn handle_online_ping(&self, mut pk: MutableBuffer) -> Result<()> {
        let ping = ConnectedPing::deserialize(pk.snapshot())?;
        let pong = ConnectedPong {
            ping_time: ping.time,
            pong_time: ping.time,
        };

        pk.clear();
        pk.reserve_to(pong.serialized_size());
        pong.serialize(&mut pk);

        self.send_raw_buffer_with_config(
            pk,
            PacketConfig {
                reliability: Reliability::Unreliable,
                priority: SendPriority::Low,
            },
        );

        Ok(())
    }
}
