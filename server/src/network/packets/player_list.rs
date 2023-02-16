use bytes::{BytesMut, BufMut};
use common::{Encodable, VResult, WriteExtensions};
use uuid::Uuid;
use crate::network::packets::login::DeviceOS;

use crate::skin::Skin;

use super::{GamePacket};

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

impl GamePacket for PlayerListAdd<'_> {
    const ID: u32 = 0x3f;
}

impl Encodable for PlayerListAdd<'_> {
    fn encode(&self) -> VResult<BytesMut> {
        tracing::debug!("{self:?}");

        let mut buffer = BytesMut::new();

        buffer.put_u8(0); // Add player.
        buffer.put_var_u32(self.entries.len() as u32);
        for entry in self.entries {
            buffer.put_uuid(&entry.uuid);
            
            buffer.put_var_i64(entry.entity_id);
            buffer.put_string(entry.username);
            buffer.put_string(&entry.xuid.to_string());
            buffer.put_string(""); // Platform chat ID.
            buffer.put_i32_le(entry.device_os as i32);
            entry.skin.encode(&mut buffer);
            buffer.put_bool(false); // Player is not a teacher.
            buffer.put_bool(entry.host);
        }

        for entry in self.entries {
            buffer.put_bool(entry.skin.is_trusted);
        }

        Ok(buffer)
    }
}

/// Removes player(s) from the client's player list.
#[derive(Debug, Clone)]
pub struct PlayerListRemove<'a> {
    pub entries: &'a [Uuid]
}

impl GamePacket for PlayerListRemove<'_> {
    const ID: u32 = 0x3f;
}

impl Encodable for PlayerListRemove<'_> {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(2 + self.entries.len() * 16);

        buffer.put_u8(1); // Remove player.
        buffer.put_var_u32(self.entries.len() as u32);
        for entry in self.entries {
            buffer.put_uuid(entry);
        }

        Ok(buffer)
    }
}
