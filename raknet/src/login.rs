use proto::raknet::{ConnectedPing, ConnectedPong, ConnectionRequest, ConnectionRequestAccepted, NewIncomingConnection};
use util::{Deserialize, Serialize};
use util::MutableBuffer;

use crate::{RaknetUser, Reliability, SendPriority, SendConfig};

impl RaknetUser {
    /// Handles a [`ConnectionRequest`] packet.
    pub fn handle_connection_request(&self, mut packet: MutableBuffer) -> anyhow::Result<()> {
        let request = ConnectionRequest::deserialize(packet.as_ref())?;
        let reply = ConnectionRequestAccepted {
            client_address: self.address,
            request_time: request.time,
        };

        packet.clear();
        packet.reserve_to(reply.serialized_size());
        reply.serialize(&mut packet)?;

        self.send_raw_buffer(packet);
        Ok(())
    }

    /// Handles a [`NewIncomingConnection`] packet.
    pub fn handle_new_incoming_connection(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        let _request = NewIncomingConnection::deserialize(packet.as_ref())?;
        Ok(())
    }

    /// Handles an [`ConnectedPing`] packet.
    pub fn handle_connected_ping(&self, mut packet: MutableBuffer) -> anyhow::Result<()> {
        let ping = ConnectedPing::deserialize(packet.as_ref())?;
        let pong = ConnectedPong {
            ping_time: ping.time,
            pong_time: ping.time,
        };

        packet.clear();
        packet.reserve_to(pong.serialized_size());
        pong.serialize(&mut packet)?;

        self.send_raw_buffer_with_config(
            packet,
            SendConfig {
                reliability: Reliability::Unreliable,
                priority: SendPriority::Low,
            },
        );

        Ok(())
    }
}
