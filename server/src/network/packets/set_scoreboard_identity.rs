use bytes::{BufMut, BytesMut};
use common::{Serialize, VResult, WriteExtensions};

use super::GamePacket;

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

impl GamePacket for SetScoreboardIdentity {
    const ID: u32 = 0x70;
}

impl Serialize for SetScoreboardIdentity {
    fn serialize(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_u8(self.action as u8);
        match self.action {
            ScoreboardIdentityAction::Add => {
                buffer.put_var_u32(self.entries.len() as u32);
                for entry in &self.entries {
                    buffer.put_var_i64(entry.entry_id);
                    buffer.put_var_i64(entry.entity_unique_id);
                }
            }
            ScoreboardIdentityAction::Clear => {
                buffer.put_var_u32(self.entries.len() as u32);
                for entry in &self.entries {
                    buffer.put_var_i64(entry.entry_id);
                }
            }
        }

        Ok(buffer)
    }
}
