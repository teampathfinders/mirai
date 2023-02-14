use bytes::{BytesMut, BufMut};
use common::{Encodable, VResult, WriteExtensions};

use crate::skin::Skin;

use super::GamePacket;

#[derive(Debug)]
pub struct UpdateSkin {
    pub uuid: u128,
    pub skin: Skin
}

impl GamePacket for UpdateSkin {
    const ID: u32 = 0x5d;
}

impl Encodable for UpdateSkin {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_u128_le(self.uuid);
        self.skin.encode(&mut buffer);
        buffer.put_string(""); // Old skin name. Unused
        buffer.put_string(""); // New skin name. Unused
        buffer.put_bool(self.skin.trusted);

        Ok(buffer)
    }
}