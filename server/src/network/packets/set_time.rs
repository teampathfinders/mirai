use bytes::BytesMut;
use common::{VResult, WriteExtensions};

use crate::network::Encodable;

use super::GamePacket;

/// Sets the current time for the client.
#[derive(Debug)]
pub struct SetTime {
    /// Current time (in ticks)
    pub time: i32,
}

impl GamePacket for SetTime {
    const ID: u32 = 0x0a;
}

impl Encodable for SetTime {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_var_i32(self.time);

        Ok(buffer)
    }
}
