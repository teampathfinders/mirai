use level::{BlockStates, SubChunk, SubChunkVersion, SubStorage};
use util::{BinaryWrite, RVec};

pub trait NetworkChunkExt {
    /// Serialises the sub chunk into a new buffer and returns it in network format.
    fn serialize_network(&self, states: &BlockStates) -> anyhow::Result<RVec> {
        let mut buffer = RVec::alloc();
        self.serialize_network_in(states, &mut buffer)?;
        Ok(buffer)
    }

    fn serialize_network_in<W>(&self, states: &BlockStates, writer: W) -> anyhow::Result<()>
    where
        W: BinaryWrite;
}

impl NetworkChunkExt for SubStorage {
    fn serialize_network_in<W>(&self, states: &BlockStates, mut writer: W) -> anyhow::Result<()>
    where
        W: BinaryWrite,
    {
        level::serialize_packed_array(&mut writer, &self.indices, self.palette.len(), true)?;

        if !self.palette.is_empty() {
            writer.write_var_i32(self.palette.len() as i32)?;
        }

        for entry in &self.palette {
            // Obtain block runtime ID of palette entry.
            let runtime_id = states.state(entry).unwrap_or(states.air());
            tracing::debug!("{}: {runtime_id}", entry.name);

            writer.write_var_i32(runtime_id as i32)?;

            // https://github.com/df-mc/dragonfly/blob/master/server/world/chunk/paletted_storage.go#L35
            // Requires new palette storage that only stores runtime IDs.
        }

        Ok(())
    }
}

impl NetworkChunkExt for SubChunk {
    /// Serialises the sub chunk into the given writer in network format.
    fn serialize_network_in<W>(&self, states: &BlockStates, mut writer: W) -> anyhow::Result<()>
    where
        W: BinaryWrite,
    {
        writer.write_u8(self.version as u8)?;
        writer.write_u8(self.layers.len() as u8)?;

        if self.version == SubChunkVersion::Limitless {
            writer.write_i8(self.index)?;
        }

        for layer in &self.layers {
            layer.serialize_network_in(states, &mut writer)?;
        }

        Ok(())
    }
}
