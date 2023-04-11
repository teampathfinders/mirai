use uuid::Uuid;

use util::{Deserialize, Result, Serialize};
use util::bytes::{BinaryWrite, MutableBuffer, SharedBuffer, BinaryRead};

use crate::network::{ConnectedPacket, Skin};

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
    fn serialize<W>(&self, writer: W) -> anyhow::Result<()>
    where
        W: BinaryWrite
    {
        writer.write_u128_le(self.uuid.as_u128())?;
        self.skin.serialize(writer)?;
        writer.write_str("")?; // Old skin name. Unused
        writer.write_str("")?; // New skin name. Unused
        writer.write_bool(self.skin.is_trusted)
    }
}

impl<'a> Deserialize<'a> for UpdateSkin<'a> {
    fn deserialize<R>(reader: R) -> anyhow::Result<Self>
    where
        R: BinaryRead<'a> + 'a
    {
        // let uuid = Uuid::from_u128(buffer.get_u128_le());
        // let skin = Skin::deserialize(&mut buffer)?;

        todo!();
        // Ok(Self {
        //     uuid, skin
        // })
    }
}
