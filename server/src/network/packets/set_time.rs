use bytes::{BytesMut, Bytes};
use common::{VResult, WriteExtensions, size_of_var};

use common::Serialize;

use super::ConnectedPacket;

/// Sets the current time for the client.
#[derive(Debug, Clone)]
pub struct SetTime {
    /// Current time (in ticks)
    pub time: i32,
}

impl ConnectedPacket for SetTime {
    const ID: u32 = 0x0a;
}

impl Serialize for SetTime {
    fn serialize(&self) -> VResult<Bytes> {
        let mut buffer = BytesMut::with_capacity(size_of_var(self.time));

        buffer.put_var_i32(self.time);

        Ok(buffer.freeze())
    }
}
