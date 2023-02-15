use bytes::{BufMut, BytesMut, Buf};
use common::{Encodable, VResult, WriteExtensions, Decodable};
use uuid::Uuid;

use crate::skin::Skin;

use super::GamePacket;

#[derive(Debug, Clone)]
pub struct UpdateSkin {
    pub uuid: Uuid,
    pub skin: Skin,
}

impl GamePacket for UpdateSkin {
    const ID: u32 = 0x5d;
}

impl Encodable for UpdateSkin {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_u128_le(self.uuid.as_u128());
        self.skin.encode(&mut buffer);
        buffer.put_string(""); // Old skin name. Unused
        buffer.put_string(""); // New skin name. Unused
        buffer.put_bool(self.skin.is_trusted);

        Ok(buffer)
    }
}

impl Decodable for UpdateSkin {
    fn decode(mut buffer: BytesMut) -> VResult<Self> {
        let uuid = Uuid::from_u128(buffer.get_u128_le());
        let skin = Skin::decode(&mut buffer)?;
        
        Ok(Self {
            uuid, skin
        })
    }
}
