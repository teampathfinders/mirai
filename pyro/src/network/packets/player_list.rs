use bytes::{BytesMut, BufMut, Bytes};
use common::{Serialize, Result, WriteExtensions, size_of_varint};
use uuid::Uuid;
use crate::network::packets::login::DeviceOS;
use crate::network::Skin;

use super::{ConnectedPacket};

#[derive(Debug, Clone)]
pub struct PlayerListAddEntry<'a> {
    /// UUID.
    pub uuid: Uuid,
    /// Unique entity ID.
    pub entity_id: i64,
    /// Username of the client.
    pub username: &'a str,
    /// XUID of the client.
    pub xuid: u64,
    /// Operating system of the client.
    pub device_os: DeviceOS,
    /// The client's skin.
    pub skin: &'a Skin,
    /// Whether the client is the host of the game.
    pub host: bool,
}

/// Adds player(s) to the client's player list.
/// 
/// This and [`PlayerListRemove`] are the same packet, but are separated here for optimisation reasons.
/// This separation allows the server to remove players from the player list without having to copy over all the player data
/// contained in [`PlayerListAddEntry`].
#[derive(Debug, Clone)]
pub struct PlayerListAdd<'a> {
    pub entries: &'a [PlayerListAddEntry<'a>],
}

impl ConnectedPacket for PlayerListAdd<'_> {
    const ID: u32 = 0x3f;
}

impl Serialize for PlayerListAdd<'_> {
    fn serialize(&self, buffer: &mut BytesMut) {
        buffer.put_u8(0); // Add player.
        buffer.put_var_u32(self.entries.len() as u32);
        for entry in self.entries {
            buffer.put_uuid(&entry.uuid);
            
            buffer.put_var_i64(entry.entity_id);
            buffer.put_string(entry.username);
            buffer.put_string(&entry.xuid.to_string());
            buffer.put_string(""); // Platform chat ID.
            buffer.put_i32_le(entry.device_os as i32);
            entry.skin.serialize(buffer);
            buffer.put_bool(false); // Player is not a teacher.
            buffer.put_bool(entry.host);
        }

        for entry in self.entries {
            buffer.put_bool(entry.skin.is_trusted);
        }
    }
}

/// Removes player(s) from the client's player list.
#[derive(Debug, Clone)]
pub struct PlayerListRemove<'a> {
    pub entries: &'a [Uuid]
}

impl ConnectedPacket for PlayerListRemove<'_> {
    const ID: u32 = 0x3f;

    fn serialized_size(&self) -> usize {
        1 + size_of_varint(self.entries.len() as u32) + 16 * self.entries.len()
    }
}

impl Serialize for PlayerListRemove<'_> {
    fn serialize(&self, buffer: &mut BytesMut) {
        buffer.put_u8(1); // Remove player.
        buffer.put_var_u32(self.entries.len() as u32);
        for entry in self.entries {
            buffer.put_uuid(entry);
        }
    }
}
