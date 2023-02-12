use bytes::BytesMut;
use common::{VResult, WriteExtensions};

use crate::network::Encodable;

use super::GamePacket;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CreditStatus {
    Start,
    End,
}

impl CreditStatus {
    pub fn encode(&self, buffer: &mut BytesMut) {
        buffer.put_var_i32(*self as i32);
    }
}

#[derive(Debug)]
pub struct ShowCredits {
    pub runtime_id: u64,
    pub status: CreditStatus,
}

impl GamePacket for ShowCredits {
    const ID: u32 = 0x4b;
}

impl Encodable for ShowCredits {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_var_u64(self.runtime_id);
        self.status.encode(&mut buffer);

        Ok(buffer)
    }
}
