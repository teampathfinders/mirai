use util::{Deserialize, Serialize};
use util::bytes::MutableBuffer;
use util::Result;

use crate::raknet::{PacketConfig, SendPriority};
use crate::raknet::ConnectedPing;
use crate::raknet::ConnectedPong;
use crate::raknet::ConnectionRequest;
use crate::raknet::ConnectionRequestAccepted;
use crate::raknet::NewIncomingConnection;
use crate::raknet::Reliability;
use crate::network::Session;

impl Session {
    /// Handles a [`ConnectionRequest`] packet.
    pub fn process_connection_request(&self, mut packet: MutableBuffer) -> anyhow::Result<()> {
        let request = ConnectionRequest::deserialize(packet.snapshot())?;
        let reply = ConnectionRequestAccepted {
            client_address: self.raknet.address,
            request_time: request.time,
        };

        packet.clear();
        packet.reserve_to(reply.serialized_size());
        reply.serialize(&mut packet)?;

        self.send_raw_buffer(packet);
        Ok(())
    }

    /// Handles a [`NewIncomingConnection`] packet.
    pub fn process_new_incoming_connection(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        let _request = NewIncomingConnection::deserialize(packet.snapshot())?;
        Ok(())
    }

    /// Handles an [`ConnectedPing`] packet.
    pub fn process_online_ping(&self, mut packet: MutableBuffer) -> anyhow::Result<()> {
        let ping = ConnectedPing::deserialize(packet.snapshot())?;
        let pong = ConnectedPong {
            ping_time: ping.time,
            pong_time: ping.time,
        };

        packet.clear();
        packet.reserve_to(pong.serialized_size());
        pong.serialize(&mut packet)?;

        self.send_raw_buffer_with_config(
            packet,
            PacketConfig {
                reliability: Reliability::Unreliable,
                priority: SendPriority::Low,
            },
        );

        Ok(())
    }
}
