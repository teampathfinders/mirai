use util::{Serialize};
use util::{BinaryWrite, size_of_varint};

use crate::bedrock::ConnectedPacket;

/// An action to perform on an identity entry.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ScoreboardIdentityAction {
    Add,
    Clear,
}

/// Describes an identity entry that can be added or removed from a scoreboard.
#[derive(Debug, Clone)]
pub struct ScoreboardIdentityEntry {
    /// Unique identifer of the entry.
    pub entry_id: i64,
    pub entity_unique_id: i64,
}

#[derive(Debug, Clone)]
pub struct SetScoreboardIdentity {
    /// Action to perform on the identity entries.
    pub action: ScoreboardIdentityAction,
    /// Affected identity entires.
    pub entries: Vec<ScoreboardIdentityEntry>,
}

impl ConnectedPacket for SetScoreboardIdentity {
    const ID: u32 = 0x70;

    fn serialized_size(&self) -> usize {
        1 + size_of_varint(self.entries.len() as u32) +
            match self.action {
                ScoreboardIdentityAction::Add => {
                    self.entries.iter().fold(
                        0, |acc, e| acc + size_of_varint(e.entry_id) + size_of_varint(e.entity_unique_id),
                    )
                }
                ScoreboardIdentityAction::Clear => {
                    self.entries.iter().fold(
                        0, |acc, e| acc + size_of_varint(e.entry_id),
                    )
                }
            }
    }
}

impl Serialize for SetScoreboardIdentity {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_u8(self.action as u8)?;
        match self.action {
            ScoreboardIdentityAction::Add => {
                writer.write_var_u32(self.entries.len() as u32)?;
                for entry in &self.entries {
                    writer.write_var_i64(entry.entry_id)?;
                    writer.write_var_i64(entry.entity_unique_id)?;
                }
            }
            ScoreboardIdentityAction::Clear => {
                writer.write_var_u32(self.entries.len() as u32)?;
                for entry in &self.entries {
                    writer.write_var_i64(entry.entry_id)?;
                }
            }
        }

        Ok(())
    }
}
