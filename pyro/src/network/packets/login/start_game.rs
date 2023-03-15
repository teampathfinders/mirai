use std::collections::HashMap;

use bytes::{BufMut, BytesMut, Bytes};
use level::Dimension;

use util::{Serialize, VarInt, VarString};
use util::{bail, Error, Result};
use util::{BlockPosition, Vector2f, Vector3f, WriteExtensions};
use util::bytes::MutableBuffer;
use crate::network::packets::{CLIENT_VERSION_STRING, Difficulty, GameMode, ConnectedPacket, GameRule};
use crate::network::packets::login::ExperimentData;

const MULTIPLAYER_CORRELATION_ID: &str = "5b39a9d6-f1a1-411a-b749-b30742f81771";

#[derive(Debug, Copy, Clone)]
#[repr(i32)]
pub enum WorldGenerator {
    OldLimited,
    Infinite,
    Flat,
    Nether,
    End,
}

impl WorldGenerator {
    pub fn serialize(&self, buffer: &mut BytesMut) {
        buffer.put_var_i32(*self as i32);
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(i32)]
pub enum PermissionLevel {
    Visitor,
    Member,
    Operator,
    Custom,
}

#[derive(Debug, Clone)]
pub struct EducationResourceURI {
    pub button_name: String,
    pub link_uri: String,
}

impl EducationResourceURI {
    pub fn encode(&self, buffer: &mut BytesMut) {
        buffer.put_string(&self.button_name);
        buffer.put_string(&self.link_uri);
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum ChatRestrictionLevel {
    None,
    Dropped,
    Disabled,
}

impl ChatRestrictionLevel {
    pub fn serialize(&self, buffer: &mut BytesMut) {
        buffer.write_le::<u8>(*self as u8);
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(i32)]
pub enum PlayerMovementType {
    ClientAuthoritative,
    ServerAuthoritative,
    ServerAuthoritativeWithRewind,
}

#[derive(Debug, Copy, Clone)]
pub struct PlayerMovementSettings {
    pub movement_type: PlayerMovementType,
    pub rewind_history_size: i32,
    pub server_authoritative_breaking: bool,
}

impl PlayerMovementSettings {
    pub fn serialized_size(&self) -> usize {
        (self.movement_type as i32).var_len() +
        self.rewind_history_size.var_len() +
        1
    }

    pub fn serialize(&self, buffer: &mut BytesMut) {
        buffer.put_var_i32(self.movement_type as i32);
        buffer.put_var_i32(self.rewind_history_size);
        buffer.write_le(self.server_authoritative_breaking);
    }
}

#[derive(Debug, Clone)]
pub struct BlockEntry {
    /// Name of the block.
    pub name: String,
    // NBT compound containing properties.
    // pub properties: nbt::Value,
}

impl BlockEntry {
    pub fn serialized_size(&self) -> usize {
        self.name.var_len() + self.properties.serialized_net_size("")
    }

    pub fn serialize(&self, buffer: &mut BytesMut) {
        buffer.put_string(&self.name);

        todo!();
        // nbt::RefTag { name: "", value: &self.properties }.serialize_net(buffer);
    }
}

#[derive(Debug, Clone)]
pub struct ItemEntry {
    /// Name of the item.
    pub name: String,
    /// Runtime ID of the item.
    /// This ID is what Minecraft uses to refer to the item.
    pub runtime_id: u16,
    /// Whether this is a custom item.
    pub component_based: bool,
}

impl ItemEntry {
    pub fn serialized_size(&self) -> usize {
        self.name.var_len() + 2 + 1
    }

    pub fn serialize(&self, buffer: &mut BytesMut) {
        buffer.put_string(&self.name);
        buffer.write_le::<u16>(self.runtime_id);
        buffer.write_le(self.component_based);
    }
}

#[derive(Debug, Copy, Clone)]
pub enum SpawnBiomeType {
    Default,
    Custom,
}

#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum BroadcastIntent {
    NoMultiplayer,
    InviteOnly,
    FriendsOnly,
    FriendsOfFriends,
    Public,
}

impl BroadcastIntent {
    pub fn serialize(&self, buffer: &mut BytesMut) {
        buffer.put_var_u32(*self as u32);
    }
}

/// The start game packet contains most of the world settings displayed in the settings menu.
#[derive(Debug, Clone)]
pub struct StartGame<'a> {
    pub entity_id: i64,
    /// Runtime ID of the client.
    pub runtime_id: u64,
    /// Current game mode of the client.
    /// This is not the same as the world game mode.
    pub game_mode: GameMode,
    /// Spawn position.
    pub position: Vector3f,
    /// Spawn rotation.
    pub rotation: Vector2f,
    /// World seed.
    /// This is displayed in the settings menu.
    pub world_seed: u64,
    pub spawn_biome_type: SpawnBiomeType,
    pub custom_biome_name: &'a str,
    /// Dimension the client spawns in.
    pub dimension: Dimension,
    /// Generator used to create the world.
    /// This is also displayed in the settings menu.
    pub generator: WorldGenerator,
    /// World game mode.
    /// The default game mode for new players.
    pub world_game_mode: GameMode,
    /// Difficulty of the game.
    pub difficulty: Difficulty,
    /// Default spawn position.
    pub world_spawn: BlockPosition,
    /// Whether achievements are disabled.
    /// This should generally be set to true for servers.
    ///
    /// According to wiki.vg, the client crashes if both achievements and commands are enabled.
    /// I couldn't reproduce this on Windows 10.
    pub achievements_disabled: bool,
    /// Whether the world is in editor mode.
    pub editor_world: bool,
    /// The time to which the daylight cycle is locked if the gamerule is set.
    pub day_cycle_lock_time: i32,
    /// Whether education edition features are enabled.
    pub education_features_enabled: bool,
    /// The intensity of the current rain.
    /// Set to 0 for no rain.
    pub rain_level: f32,
    /// The intensity of the current thunderstorm.
    /// Set to 0 to for no thunderstorm.
    pub lightning_level: f32,
    pub confirmed_platform_locked_content: bool,
    /// Whether to broadcast to LAN.
    pub broadcast_to_lan: bool,
    pub xbox_broadcast_intent: BroadcastIntent,
    pub platform_broadcast_intent: BroadcastIntent,
    /// Whether to enable commands.
    /// If this is disabled, the client will not allow the player to send commands under any
    /// circumstance.
    pub enable_commands: bool,
    /// Whether texture packs are required.
    /// This doesn't really seem to have a function other than displaying the force pack setting in the settings menu.
    ///
    /// Whether packs are actually required has already been specified in the resource pack packets.
    pub texture_packs_required: bool,
    /// List of game rules.
    /// Only modified game rules have to be sent.
    /// Game rules that are not sent in the start game packet, will be set to their default values.
    pub game_rules: &'a [GameRule],
    /// Experiments used by the server.
    /// This is a visual option, since the experiments have already been specified in a resource pack packet.
    pub experiments: &'a [ExperimentData<'a>],
    /// Whether experiments have previously been enabled.
    pub experiments_previously_enabled: bool,
    /// Whether the bonus chest is enabled.
    /// This is only a visual thing shown in the settings menu.
    pub bonus_chest_enabled: bool,
    /// Whether the starter map is enabled.
    /// This generally should be left disabled as the client will otherwise force itself to have a map in its inventory.
    pub starter_map_enabled: bool,
    /// Permission level of the client: visitor, member, operator or custom.
    pub permission_level: PermissionLevel,
    /// Simulation distance of the server.
    /// This is only a visual thing shown in the settings menu.
    pub server_chunk_tick_range: i32,

    pub has_locked_behavior_pack: bool,
    pub has_locked_resource_pack: bool,
    pub is_from_locked_world_template: bool,
    pub use_msa_gamertags_only: bool,
    pub is_from_world_template: bool,
    pub is_world_template_option_locked: bool,
    pub only_spawn_v1_villagers: bool,
    pub persona_disabled: bool,
    pub custom_skins_disabled: bool,
    pub emote_chat_muted: bool,
    /// Version of the game from which vanilla features will be used.
    // pub base_game_version: String,
    pub limited_world_width: i32,
    pub limited_world_height: i32,
    // pub has_new_nether: bool,
    pub force_experimental_gameplay: bool,
    pub chat_restriction_level: ChatRestrictionLevel,
    pub disable_player_interactions: bool,
    pub level_id: &'a str,
    /// Name of the world.
    /// This is shown in the pause menu above the player list, and the settings menu.
    pub level_name: &'a str,
    pub template_content_identity: &'a str,
    pub movement_settings: PlayerMovementSettings,
    /// Current time.
    pub time: i64,
    pub enchantment_seed: i32,
    pub block_properties: &'a [BlockEntry],
    pub item_properties: &'a [ItemEntry],
    /// Whether inventory transactions are server authoritative.
    pub server_authoritative_inventory: bool,
    /// Version of the game that the server is running.
    pub game_version: &'a str,
    // pub property_data: nbt::Value,
    pub server_block_state_checksum: u64,
    pub world_template_id: u128,
    /// Client side generation allows the client to generate its own chunks without the server having to send them over.
    pub client_side_generation: bool,
}

impl ConnectedPacket for StartGame<'_> {
    const ID: u32 = 0x0b;

    fn serialized_size(&self) -> usize {
        // self.entity_id.var_len() +
        // self.runtime_id.var_len() +
        // (self.game_mode as i32).var_len() +
        // 3 * 4 + 
        // 3 * 4 +
        // 8 +
        // 2 +
        // self.custom_biome_name.var_len() +
        // (self.dimension as u32).var_len() +
        // (self.generator as i32).var_len() +
        // (self.world_game_mode as i32).var_len() +
        // (self.difficulty as i32).var_len() +
        // self.world_spawn.serialized_size() +
        // 1 +
        // 1 +
        // self.day_cycle_lock_time.var_len() +
        // 0.var_len() +
        // 1 +
        // "".var_len() +
        // 4 +
        // 4 +
        // 1 +
        // 1 +
        // 1 +
        // (self.xbox_broadcast_intent as u32).var_len() +
        // (self.platform_broadcast_intent as u32).var_len() +
        // 1 +
        // 1 +
        // (self.game_rules.len() as u32).var_len() +
        // self.game_rules.iter().fold(0, |acc, r| acc + r.serialized_size()) +
        // 4 +
        // self.experiments.iter().fold(0, |acc, e| acc + e.serialized_size()) +
        // 1 +
        // 1 +
        // 1 +
        // (self.permission_level as i32).var_len() +
        // 4 +
        // 1 +
        // 1 +
        // 1 +
        // 1 +
        // 1 +
        // 1 +
        // 1 +
        // 1 +
        // 1 +
        // 1 +
        // CLIENT_VERSION_STRING.var_len() +
        // 4 +
        // 4 +
        // 1 +
        // "".var_len() +
        // "".var_len() +
        // 1 +
        // 1 +
        // 1 +
        // self.level_id.var_len() +
        // self.level_name.var_len() +
        // self.template_content_identity.var_len() +
        // 1 +
        // self.movement_settings.serialized_size() +
        // 8 +
        // self.enchantment_seed.var_len() +
        // (self.block_properties.len() as u32).var_len() +
        // self.block_properties.iter().fold(0, |acc, p| acc + p.serialized_size()) +
        // (self.item_properties.len() as u32).var_len() +
        // self.item_properties.iter().fold(0, |acc, p| acc + p.serialized_size()) +
        // MULTIPLAYER_CORRELATION_ID.var_len() +
        // 1 +
        // CLIENT_VERSION_STRING.var_len() +
        // self.property_data.serialized_net_size("") +
        // 8 +
        // 16 +
        1
    }
}

impl Serialize for StartGame<'_> {
    fn serialize(&self, buffer: &mut MutableBuffer) {
        buffer.put_var_i64(self.entity_id);
        buffer.put_var_u64(self.runtime_id);
        buffer.put_var_i32(self.game_mode as i32);
        self.position.serialize(buffer);
        self.rotation.serialize(buffer);
        buffer.put_u64_le(self.world_seed);
        buffer.write_le::<i16>(self.spawn_biome_type as i16);
        buffer.put_string(self.custom_biome_name);
        buffer.put_var_u32(self.dimension as u32);
        buffer.put_var_i32(self.generator as i32);
        buffer.put_var_i32(self.world_game_mode as i32);
        buffer.put_var_i32(self.difficulty as i32);
        buffer.put_block_pos(&self.world_spawn);

        buffer.write_le::<bool>(self.achievements_disabled);
        buffer.write_le::<bool>(self.editor_world);
        buffer.put_var_i32(self.day_cycle_lock_time);
        buffer.put_var_i32(0); // Education offer.
        buffer.write_le::<bool>(self.education_features_enabled);
        buffer.put_string(""); // Education product ID.
        buffer.write_le::<f32>(self.rain_level);
        buffer.write_le::<f32>(self.lightning_level);
        buffer.write_le::<bool>(self.confirmed_platform_locked_content);
        buffer.write_le::<bool>(true); // Whether the game is multiplayer, must always be true for servers.
        buffer.write_le::<bool>(self.broadcast_to_lan);
        self.xbox_broadcast_intent.serialize(buffer);
        self.platform_broadcast_intent.serialize(buffer);
        buffer.write_le::<bool>(self.enable_commands);
        buffer.write_le::<bool>(self.texture_packs_required);

        buffer.put_var_u32(self.game_rules.len() as u32);
        for rule in self.game_rules {
            rule.serialize(buffer);
        }

        buffer.write_le::<u32>(self.experiments.len() as u32);
        for experiment in self.experiments {
            experiment.serialize(buffer);
        }

        buffer.write_le::<bool>(self.experiments_previously_enabled);
        buffer.write_le::<bool>(self.bonus_chest_enabled);
        buffer.write_le::<bool>(self.starter_map_enabled);
        buffer.put_var_i32(self.permission_level as i32);
        buffer.write_le::<i32>(self.server_chunk_tick_range);
        buffer.write_le::<bool>(self.has_locked_behavior_pack);
        buffer.write_le::<bool>(self.has_locked_resource_pack);
        buffer.write_le::<bool>(self.is_from_locked_world_template);
        buffer.write_le::<bool>(self.use_msa_gamertags_only);
        buffer.write_le::<bool>(self.is_from_world_template);
        buffer.write_le::<bool>(self.is_world_template_option_locked);
        buffer.write_le::<bool>(self.only_spawn_v1_villagers);
        buffer.write_le::<bool>(self.persona_disabled);
        buffer.write_le::<bool>(self.custom_skins_disabled);
        buffer.write_le::<bool>(self.emote_chat_muted);
        buffer.put_string(CLIENT_VERSION_STRING); // Base game version
        buffer.write_le::<i32>(self.limited_world_width);
        buffer.write_le::<i32>(self.limited_world_height);
        buffer.write_le::<bool>(true); // Use new nether
        buffer.put_string("");
        buffer.put_string("");
        buffer.write_le::<bool>(self.force_experimental_gameplay);
        self.chat_restriction_level.serialize(buffer);
        buffer.write_le::<bool>(self.disable_player_interactions);
        buffer.put_string(self.level_id);
        buffer.put_string(self.level_name);
        buffer.put_string(self.template_content_identity);
        buffer.write_le::<bool>(false); // Game is not a trial.
        self.movement_settings.serialize(buffer);
        buffer.write_le::<i64>(self.time);
        buffer.put_var_i32(self.enchantment_seed);

        buffer.put_var_u32(self.block_properties.len() as u32);
        for block in self.block_properties {
            block.serialize(buffer);
        }

        buffer.put_var_u32(self.item_properties.len() as u32);
        for item in self.item_properties {
            item.serialize(buffer);
        }

        // Random multiplayer correlation UUID.
        buffer.put_string(MULTIPLAYER_CORRELATION_ID);

        buffer.write_le::<bool>(self.server_authoritative_inventory);
        buffer.put_string(CLIENT_VERSION_STRING); // Game version

        todo!();
        // nbt::serialize_net("", &self.property_data, buffer);

        buffer.write_le::<u64>(self.server_block_state_checksum);
        buffer.write_le::<u128>(self.world_template_id);
        buffer.write_le::<bool>(self.client_side_generation);
    }
}
