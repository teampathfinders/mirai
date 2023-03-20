use util::Serialize;
use util::bytes::{BinaryWrite, MutableBuffer, size_of_varint, VarString};
use util::Result;

use crate::ConnectedPacket;

#[derive(Debug, Clone)]
pub struct ExperimentData<'a> {
    pub name: &'a str,
    pub enabled: bool,
}

impl ExperimentData<'_> {
    pub fn serialized_size(&self) -> usize {
        self.name.var_len() + 1
    }

    pub fn serialize(&self, buffer: &mut MutableBuffer) -> Result<()> {
        buffer.write_str(&self.name)?;
        buffer.write_bool(self.enabled)
    }
}

#[derive(Debug, Clone)]
pub struct ResourcePackStackEntry<'a> {
    pub pack_id: &'a str,
    pub pack_version: &'a str,
    pub subpack_name: &'a str,
}

impl ResourcePackStackEntry<'_> {
    pub fn serialized_size(&self) -> usize {
        self.pack_id.var_len() +
            self.pack_version.var_len() +
            self.subpack_name.var_len()
    }

    pub fn serialize(&self, buffer: &mut MutableBuffer) -> Result<()> {
        buffer.write_str(&self.pack_id)?;
        buffer.write_str(&self.pack_version)?;
        buffer.write_str(&self.subpack_name)
    }
}

#[derive(Debug)]
pub struct ResourcePackStack<'a> {
    pub forced_to_accept: bool,
    pub resource_packs: &'a [ResourcePackStackEntry<'a>],
    pub behavior_packs: &'a [ResourcePackStackEntry<'a>],
    pub game_version: &'a str,
    pub experiments: &'a [ExperimentData<'a>],
    pub experiments_previously_toggled: bool,
}

impl ConnectedPacket for ResourcePackStack<'_> {
    const ID: u32 = 0x07;

    fn serialized_size(&self) -> usize {
        1 +
            size_of_varint(self.resource_packs.len() as u32) +
            self.resource_packs.iter().fold(0, |acc, p| acc + p.serialized_size()) +

            size_of_varint(self.behavior_packs.len() as u32) +
            self.behavior_packs.iter().fold(0, |acc, p| acc + p.serialized_size()) +

            size_of_varint(self.game_version.len() as u32) + self.game_version.len() +
            4 + self.experiments.iter().fold(0, |acc, e| acc + e.serialized_size()) +
            1
    }
}

impl Serialize for ResourcePackStack<'_> {
    fn serialize(&self, buffer: &mut MutableBuffer) -> Result<()> {
        buffer.write_bool(self.forced_to_accept)?;

        buffer.write_var_u32(self.resource_packs.len() as u32)?;
        for pack in self.resource_packs {
            pack.serialize(buffer)?;
        }

        buffer.write_var_u32(self.behavior_packs.len() as u32)?;
        for pack in self.behavior_packs {
            pack.serialize(buffer)?;
        }

        buffer.write_str(self.game_version)?;

        buffer.write_u32_be(self.experiments.len() as u32)?;
        for experiment in self.experiments {
            experiment.serialize(buffer)?;
        }

        buffer.write_bool(self.experiments_previously_toggled)
    }
}
