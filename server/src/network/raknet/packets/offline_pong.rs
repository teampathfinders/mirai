use bytes::Bytes;
use bytes::{BufMut, BytesMut};

use crate::network::raknet::OFFLINE_MESSAGE_DATA;
use common::Serialize;
use common::VResult;
use common::WriteExtensions;

/// Response to [`OfflinePing`](super::offline_ping::OfflinePing).
#[derive(Debug)]
pub struct OfflinePong<'a> {
    /// Timestamp of when the ping was sent.
    /// This should be given the same value as [`OfflinePing::time`](super::offline_ping::OfflinePing::time).
    pub time: i64,
    /// Randomised GUID of the server.
    /// Corresponds to [`ServerInstance::guid`](crate::ServerInstance::guid)
    pub server_guid: i64,
    /// Contains the info to be displayed in the server banner in the server tab.
    /// Corresponds to [`ServerInstance::metadata`](crate::ServerInstance::metadata)
    pub metadata: &'a str,
}

impl OfflinePong<'_> {
    /// Unique identifier of this packet.
    pub const ID: u8 = 0x1c;
}

impl Serialize for OfflinePong<'_> {
    fn serialize(&self) -> VResult<Bytes> {
        let mut buffer =
            BytesMut::with_capacity(1 + 8 + 8 + 16 + 2 + self.metadata.len());

        buffer.put_u8(Self::ID);
        buffer.put_i64(self.time);
        buffer.put_i64(self.server_guid);
        buffer.put(OFFLINE_MESSAGE_DATA);
        buffer.put_raknet_string(self.metadata);

        Ok(buffer.freeze())
    }
}
