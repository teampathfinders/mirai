use bytes::{BytesMut, BufMut};
use common::{Encodable, VResult, WriteExtensions};
use uuid::Uuid;

use crate::skin::Skin;

use super::{BuildPlatform, GamePacket};

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
    pub build_platform: BuildPlatform,
    /// The client's skin.
    pub skin: &'a Skin,
    /// Whether the client is a teacher.
    pub teacher: bool,
    /// Whether the client is the host of the game.
    pub host: bool,
}

#[derive(Debug, Clone)]
pub struct PlayerListAdd<'a> {
    pub entries: &'a [PlayerListAddEntry<'a>],
}

impl GamePacket for PlayerListAdd<'_> {
    const ID: u32 = 0x3f;
}

impl Encodable for PlayerListAdd<'_> {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_u8(0); // Add player.
        buffer.put_var_u32(self.entries.len() as u32);
        for entry in self.entries {
            buffer.put_u128_le(entry.uuid.as_u128());
            buffer.put_var_i64(entry.entity_id);
            buffer.put_string(entry.username);
            buffer.put_string(&entry.xuid.to_string());
            buffer.put_string(""); // Platform chat ID.
            buffer.put_i32_le(entry.build_platform as i32);
            entry.skin.encode(&mut buffer);
            buffer.put_bool(entry.teacher);
            buffer.put_bool(entry.host);
        }

        Ok(buffer)
    }
}

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
            buffer.put_u128_le(entry.as_u128());
        }

        Ok(buffer)
    }
}