use bytes::{BufMut, BytesMut, Buf};
use common::{Serialize, VResult, WriteExtensions, Deserialize};
use uuid::Uuid;
use crate::network::Skin;

use super::ConnectedPacket;

#[derive(Debug, Clone)]
pub struct UpdateSkin {
    pub uuid: Uuid,
    pub skin: Skin,
}

impl ConnectedPacket for UpdateSkin {
    const ID: u32 = 0x5d;
}

impl Serialize for UpdateSkin {
    fn serialize(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_u128_le(self.uuid.as_u128());
        self.skin.encode(&mut buffer);
        buffer.put_string(""); // Old skin name. Unused
        buffer.put_string(""); // New skin name. Unused
        buffer.put_bool(self.skin.is_trusted);

        Ok(buffer)
    }
}

impl Deserialize for UpdateSkin {
    fn deserialize(mut buffer: BytesMut) -> VResult<Self> {
        let uuid = Uuid::from_u128(buffer.get_u128_le());
        let skin = Skin::decode(&mut buffer)?;
        
        Ok(Self {
            uuid, skin
        })
    }
}
