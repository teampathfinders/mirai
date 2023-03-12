use bytes::{BufMut, BytesMut, Buf, Bytes};
use common::{Serialize, Result, WriteExtensions, Deserialize};
use uuid::Uuid;
use crate::network::{Skin, packets::ConnectedPacket};

#[derive(Debug, Clone)]
pub struct UpdateSkin {
    pub uuid: Uuid,
    pub skin: Skin,
}

impl ConnectedPacket for UpdateSkin {
    const ID: u32 = 0x5d;

    fn serialized_size(&self) -> usize {
        todo!();
    }
}

impl Serialize for UpdateSkin {
    fn serialize(&self, buffer: &mut BytesMut) {
        buffer.put_u128_le(self.uuid.as_u128());
        self.skin.serialize(buffer);
        buffer.put_string(""); // Old skin name. Unused
        buffer.put_string(""); // New skin name. Unused
        buffer.put_bool(self.skin.is_trusted);
    }
}

impl Deserialize for UpdateSkin {
    fn deserialize(mut buffer: Bytes) -> Result<Self> {
        let uuid = Uuid::from_u128(buffer.get_u128_le());
        let skin = Skin::deserialize(&mut buffer)?;
        
        Ok(Self {
            uuid, skin
        })
    }
}
