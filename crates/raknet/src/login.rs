use proto::raknet::{ConnectedPing, ConnectedPong, ConnectionRequest, ConnectionRequestAccepted, NewIncomingConnection};
use util::{RVec, Deserialize, ReserveTo, Serialize};

use crate::{RakNetClient, Reliability, SendPriority, SendConfig};

impl RakNetClient {
    /// Handles a [`ConnectionRequest`] packet.
    pub fn handle_connection_request(&self, mut packet: RVec) -> anyhow::Result<()> {
        let request = ConnectionRequest::deserialize(packet.as_ref())?;

        #[cfg(trace_raknet)]
        tracing::debug!("{request:?}");

        let reply = ConnectionRequestAccepted {
            client_address: self.address,
            request_time: request.time,
        };

        packet.clear();
        packet.reserve_to(reply.size_hint());
        reply.serialize_into(&mut packet)?;

        self.send_raw_buffer(packet);
        Ok(())
    }

    /// Handles a [`NewIncomingConnection`] packet.
    pub fn handle_new_incoming_connection(&self, packet: RVec) -> anyhow::Result<()> {
        let _request = NewIncomingConnection::deserialize(packet.as_ref())?;

        #[cfg(trace_raknet)]
        tracing::debug!("{_request:?}");

        Ok(())
    }

    /// Handles an [`ConnectedPing`] packet.
    pub fn handle_connected_ping(&self, mut packet: RVec) -> anyhow::Result<()> {
        let ping = ConnectedPing::deserialize(packet.as_ref())?;

        #[cfg(trace_raknet)]
        tracing::debug!("{ping:?}");

        let pong = ConnectedPong {
            ping_time: ping.time,
            pong_time: ping.time,
        }; 

        packet.clear();
        packet.reserve_to(pong.size_hint());
        pong.serialize_into(&mut packet)?;

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
