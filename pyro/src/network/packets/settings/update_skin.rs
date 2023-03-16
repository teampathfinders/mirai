
use util::{Serialize, Result, Deserialize};
use uuid::Uuid;
use util::bytes::{BinaryWriter, MutableBuffer, SharedBuffer};
use crate::{Skin, ConnectedPacket};

#[derive(Debug, Clone)]
pub struct UpdateSkin<'a> {
    pub uuid: Uuid,
    pub skin: &'a Skin,
}

impl<'a> ConnectedPacket for UpdateSkin<'a> {
    const ID: u32 = 0x5d;

    fn serialized_size(&self) -> usize {
        todo!();
    }
}

impl<'a> Serialize for UpdateSkin<'a> {
    fn serialize(&self, buffer: &mut MutableBuffer) -> Result<()> {
        buffer.write_u128_le(self.uuid.as_u128());
        self.skin.serialize(buffer);
        buffer.write_str(""); // Old skin name. Unused
        buffer.write_str(""); // New skin name. Unused
        buffer.write_bool(self.skin.is_trusted);

        Ok(())
    }
}

impl<'a> Deserialize<'a> for UpdateSkin<'a> {
    fn deserialize(_buffer: SharedBuffer) -> Result<Self> {
        // let uuid = Uuid::from_u128(buffer.get_u128_le());
        // let skin = Skin::deserialize(&mut buffer)?;

        todo!();
        // Ok(Self {
        //     uuid, skin
        // })
    }
}
