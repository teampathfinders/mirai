use crate::skin::Skin;

use super::BuildPlatform;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PlayerListAction {
    Add,
    Remove,
}

#[derive(Debug)]
pub struct PlayerListEntry {
    pub uuid: String,
    pub entity_id: i64,
    pub username: String,
    pub xuid: String,
    pub platform_chat_id: String,
    pub build_platform: BuildPlatform,
    pub skin: Skin,
    pub teacher: bool,
    pub host: bool,
}

#[derive(Debug)]
pub struct PlayerList<'a> {
    pub action: PlayerListAction,
    pub entries: &'a [PlayerListEntry],
}
