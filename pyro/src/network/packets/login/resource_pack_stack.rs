use bytes::Bytes;
use bytes::{BufMut, BytesMut};

use crate::network::packets::ConnectedPacket;
use util::{Serialize, size_of_varint, size_of_string, VarString};
use util::bytes::WriteBuffer;
use util::Result;
use util::WriteExtensions;

#[derive(Debug, Clone)]
pub struct ExperimentData<'a> {
    pub name: &'a str,
    pub enabled: bool,
}

impl ExperimentData<'_> {
    pub fn serialized_size(&self) -> usize {
        self.name.var_len() + 1
    }

    pub fn serialize(&self, buffer: &mut WriteBuffer) {
        buffer.put_string(&self.name);
        buffer.write_le::<bool>(self.enabled);
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

    pub fn serialize(&self, buffer: &mut WriteBuffer) {
        buffer.put_string(&self.pack_id);
        buffer.put_string(&self.pack_version);
        buffer.put_string(&self.subpack_name);
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
    fn serialize(&self, buffer: &mut WriteBuffer) {
        buffer.write_le::<bool>(self.forced_to_accept);

        buffer.put_var_u32(self.resource_packs.len() as u32);
        for pack in self.resource_packs {
            pack.serialize(buffer);
        }

        buffer.put_var_u32(self.behavior_packs.len() as u32);
        for pack in self.behavior_packs {
            pack.serialize(buffer);
        }

        buffer.put_string(self.game_version);

        buffer.write_be::<u32>()(self.experiments.len() as u32);
        for experiment in self.experiments {
            experiment.serialize(buffer);
        }

        buffer.write_le::<bool>(self.experiments_previously_toggled);
    }
}
