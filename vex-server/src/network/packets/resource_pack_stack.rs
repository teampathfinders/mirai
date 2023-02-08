use bytes::{BufMut, BytesMut};

use vex_common::{Encodable, VResult, WriteExtensions};

use crate::network::Encodable;
use crate::network::packets::GamePacket;
use crate::util::WriteExtensions;

#[derive(Debug)]
pub struct ExperimentData {
    pub name: String,
    pub enabled: bool,
}

impl ExperimentData {
    pub fn encode(&self, buffer: &mut BytesMut) {
        buffer.put_string(&self.name);
        buffer.put_bool(self.enabled);
    }
}

#[derive(Debug)]
pub struct ResourcePackStack {
    pub forced_to_accept: bool,
    pub resource_packs: Vec<ResourcePackStackEntry>,
    pub behavior_packs: Vec<ResourcePackStackEntry>,
    pub game_version: String,
    pub experiments: Vec<ExperimentData>,
    pub experiments_previously_toggled: bool,
}

#[derive(Debug)]
pub struct ResourcePackStackEntry {
    pub pack_id: String,
    pub pack_version: String,
    pub subpack_name: String,
}

impl ResourcePackStackEntry {
    pub fn encode(&self, buffer: &mut BytesMut) {
        buffer.put_string(&self.pack_id);
        buffer.put_string(&self.pack_version);
        buffer.put_string(&self.subpack_name);
    }
}

#[derive(Debug)]
pub struct BehaviorPackEntry {
    pub pack_id: String,
    pub pack_version: String,
    pub subpack_name: String,
}

impl GamePacket for ResourcePackStack {
    const ID: u32 = 0x07;
}

impl Encodable for ResourcePackStack {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_bool(self.forced_to_accept);

        buffer.put_var_u32(self.resource_packs.len() as u32);
        for pack in &self.resource_packs {
            pack.encode(&mut buffer);
        }

        buffer.put_var_u32(self.behavior_packs.len() as u32);
        for pack in &self.behavior_packs {
            pack.encode(&mut buffer);
        }

        buffer.put_string(&self.game_version);

        buffer.put_u32(self.experiments.len() as u32);
        for experiment in &self.experiments {
            experiment.encode(&mut buffer);
        }

        buffer.put_bool(self.experiments_previously_toggled);

        Ok(buffer)
    }
}
