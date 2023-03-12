use bytes::{BytesMut, Bytes};
use common::{Result, WriteExtensions, size_of_varint};

use common::Serialize;

use crate::network::packets::ConnectedPacket;

/// Sets the current time for the client.
#[derive(Debug, Clone)]
pub struct SetTime {
    /// Current time (in ticks)
    pub time: i32,
}

impl ConnectedPacket for SetTime {
    const ID: u32 = 0x0a;

    fn serialized_size(&self) -> usize {
        size_of_varint(self.time)
    }
}

impl Serialize for SetTime {
    fn serialize(&self, buffer: &mut BytesMut) {
        buffer.put_var_i32(self.time);
    }
}
