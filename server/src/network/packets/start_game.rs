use std::collections::HashMap;

use bytes::{BufMut, BytesMut};

use common::VResult;
use crate::network::Encodable;
use crate::network::packets::{ExperimentData, GamePacket};
use common::{BlockPosition, Vector2f, Vector3f, WriteExtensions};

use super::CLIENT_VERSION_STRING;

#[derive(Debug, Copy, Clone)]
pub enum GameMode {
    Survival = 0,
    Creative = 1,
    Adventure = 2,
    Spectator = 3,
    /// Sets the player's game mode to the world default.
    WorldDefault = 5,
}

impl GameMode {
    pub fn encode(&self, buffer: &mut BytesMut) {
        buffer.put_var_i32(*self as i32);
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Dimension {
    Overworld,
    Nether,
    End,
}

impl Dimension {
    pub fn encode(&self, buffer: &mut BytesMut) {
        buffer.put_var_u32(*self as u32);
    }
}

#[derive(Debug, Copy, Clone)]
pub enum WorldGenerator {
    OldLimited,
    Infinite,
    Flat,
    Nether,
    End,
}

impl WorldGenerator {
    pub fn encode(&self, buffer: &mut BytesMut) {
        buffer.put_var_i32(*self as i32);
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

impl Difficulty {
    pub fn encode(&self, buffer: &mut BytesMut) {
        buffer.put_var_i32(*self as i32);
    }
}

#[derive(Debug, Clone)]
pub enum GameRule {}

impl GameRule {
    fn encode(&self, buffer: &mut BytesMut) {
        todo!();
    }
}

#[derive(Debug, Copy, Clone)]
pub enum PermissionLevel {
    Visitor,
    Member,
    Operator,
    Custom,
}

impl PermissionLevel {
    pub fn encode(&self, buffer: &mut BytesMut) {
        buffer.put_var_i32(*self as i32);
    }
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
pub enum ChatRestrictionLevel {
    None,
    Dropped,
    Disabled,
}

impl ChatRestrictionLevel {
    pub fn encode(&self, buffer: &mut BytesMut) {
        buffer.put_u8(*self as u8);
    }
}

#[derive(Debug, Copy, Clone)]
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
    pub fn encode(&self, buffer: &mut BytesMut) {
        buffer.put_var_i32(self.movement_type as i32);
        buffer.put_var_i32(self.rewind_history_size);
        buffer.put_bool(self.server_authoritative_breaking);
    }
}

#[derive(Debug, Clone)]
pub struct BlockEntry {
    /// Name of the block.
    pub name: String,
    /// NBT compound containing properties.
    pub properties: nbt::Value,
}

impl BlockEntry {
    pub fn encode(&self, buffer: &mut BytesMut) {
        buffer.put_string(&self.name);

        nbt::RefTag {
            name: "",
            value: &self.properties,
        }.encode_with_net(buffer);
    }
}

#[derive(Debug, Clone)]
pub struct ItemEntry {
    pub name: String,
    pub runtime_id: u16,
    pub component_based: bool,
}

impl ItemEntry {
    pub fn encode(&self, buffer: &mut BytesMut) {
        buffer.put_string(&self.name);
        buffer.put_u16(self.runtime_id);
        buffer.put_bool(self.component_based);
    }
}

#[derive(Debug, Copy, Clone)]
pub enum SpawnBiomeType {
    Default,
    Custom
}

impl SpawnBiomeType {
    pub fn encode(&self, buffer: &mut BytesMut) {
        buffer.put_i16(*self as i16);
    }
}

#[derive(Debug, Copy, Clone)]
pub enum BroadcastIntent {
    NoMultiplayer,
    InviteOnly,
    FriendsOnly,
    FriendsOfFriends,
    Public
}

impl BroadcastIntent {
    pub fn encode(&self, buffer: &mut BytesMut) {
        buffer.put_var_u32(*self as u32);
    }
}

#[derive(Debug)]
pub struct StartGame {
    pub entity_id: i64,
    pub runtime_id: u64,
    pub gamemode: GameMode,
    pub position: Vector3f,
    pub rotation: Vector2f,
    pub world_seed: u64,
    pub spawn_biome_type: SpawnBiomeType,
    pub custom_biome_name: String,
    pub dimension: Dimension,
    pub generator: WorldGenerator,
    pub world_gamemode: GameMode,
    pub difficulty: Difficulty,
    pub world_spawn: BlockPosition,
    pub achievements_disabled: bool,
    pub editor_world: bool,
    pub day_cycle_lock_time: i32,
    pub education_offer: i32,
    pub education_features_enabled: bool,
    pub education_production_id: String,
    pub rain_level: f32,
    pub lightning_level: f32,
    pub confirmed_platform_locked_content: bool,
    pub broadcast_to_lan: bool,
    pub xbox_broadcast_intent: BroadcastIntent,
    pub platform_broadcast_intent: BroadcastIntent,
    /// Whether to enable commands.
    /// If this is disabled, the client will not allow the player to send commands under any
    /// circumstance.
    pub enable_commands: bool,
    pub texture_packs_required: bool,
    pub gamerules: Vec<GameRule>,
    pub experiments: Vec<ExperimentData>,
    pub experiments_previously_enabled: bool,
    pub bonus_chest_enabled: bool,
    pub starter_map_enabled: bool,
    pub permission_level: PermissionLevel,
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
    pub level_id: String,
    pub level_name: String,
    pub template_content_identity: String,
    pub is_trial: bool,
    pub movement_settings: PlayerMovementSettings,
    pub time: i64,
    pub enchantment_seed: i32,
    pub block_properties: Vec<BlockEntry>,
    pub item_properties: Vec<ItemEntry>,
    pub multiplayer_correlation_id: String,
    pub server_authoritative_inventory: bool,
    pub game_version: String,
    pub property_data: nbt::Value,
    pub server_block_state_checksum: u64,
    pub world_template_id: u128,
    pub client_side_generation: bool,
}

impl GamePacket for StartGame {
    const ID: u32 = 0x0B;
}

impl Encodable for StartGame {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_var_i64(self.entity_id);
        buffer.put_var_u64(self.runtime_id);
        self.gamemode.encode(&mut buffer);
        self.position.encode(&mut buffer);
        self.rotation.encode(&mut buffer);
        buffer.put_u64(self.world_seed);
        self.spawn_biome_type.encode(&mut buffer);
        buffer.put_string(&self.custom_biome_name);
        self.dimension.encode(&mut buffer);
        self.generator.encode(&mut buffer);
        self.world_gamemode.encode(&mut buffer);
        self.difficulty.encode(&mut buffer);
        self.world_spawn.encode(&mut buffer);

        buffer.put_bool(self.achievements_disabled);
        buffer.put_bool(self.editor_world);
        buffer.put_var_i32(self.day_cycle_lock_time);
        buffer.put_var_i32(self.education_offer);
        buffer.put_bool(self.education_features_enabled);
        buffer.put_string("");
        buffer.put_f32(self.rain_level);
        buffer.put_f32(self.lightning_level);
        buffer.put_bool(self.confirmed_platform_locked_content);
        buffer.put_bool(true); // Whether the game is multiplayer. Must always be true.
        buffer.put_bool(self.broadcast_to_lan);
        self.xbox_broadcast_intent.encode(&mut buffer);
        self.platform_broadcast_intent.encode(&mut buffer);
        buffer.put_bool(self.enable_commands);
        buffer.put_bool(self.texture_packs_required);

        buffer.put_var_u32(self.gamerules.len() as u32);
        for rule in &self.gamerules {
            rule.encode(&mut buffer);
        }

        buffer.put_u32(self.experiments.len() as u32);
        for experiment in &self.experiments {
            experiment.encode(&mut buffer);
        }

        buffer.put_bool(self.experiments_previously_enabled);
        buffer.put_bool(self.bonus_chest_enabled);
        buffer.put_bool(self.starter_map_enabled);
        self.permission_level.encode(&mut buffer);
        buffer.put_i32(self.server_chunk_tick_range);
        buffer.put_bool(self.has_locked_behavior_pack);
        buffer.put_bool(self.has_locked_resource_pack);
        buffer.put_bool(self.is_from_locked_world_template);
        buffer.put_bool(self.use_msa_gamertags_only);
        buffer.put_bool(self.is_from_world_template);
        buffer.put_bool(self.is_world_template_option_locked);
        buffer.put_bool(self.only_spawn_v1_villagers);
        buffer.put_bool(self.persona_disabled);
        buffer.put_bool(self.custom_skins_disabled);
        buffer.put_bool(self.emote_chat_muted);
        buffer.put_string(CLIENT_VERSION_STRING); // Base game version
        buffer.put_i32(self.limited_world_width);
        buffer.put_i32(self.limited_world_height);
        buffer.put_bool(true); // Use new nether
        buffer.put_string("");
        buffer.put_string("");
        buffer.put_bool(self.force_experimental_gameplay);
        self.chat_restriction_level.encode(&mut buffer);
        buffer.put_bool(self.disable_player_interactions);
        buffer.put_string(&self.level_id);
        buffer.put_string(&self.level_name);
        buffer.put_string(&self.template_content_identity);
        buffer.put_bool(self.is_trial);
        self.movement_settings.encode(&mut buffer);
        buffer.put_i64(self.time);
        buffer.put_var_i32(self.enchantment_seed);

        buffer.put_var_u32(self.block_properties.len() as u32);
        for block in &self.block_properties {
            block.encode(&mut buffer);
        }

        buffer.put_var_u32(self.item_properties.len() as u32);
        for item in &self.item_properties {
            item.encode(&mut buffer);
        }

        // Random multiplayer correlation UUID.
        buffer.put_string("5b39a9d6-f1a1-411a-b749-b30742f81771");

        buffer.put_bool(self.server_authoritative_inventory);
        buffer.put_string(CLIENT_VERSION_STRING); // Game version

        nbt::RefTag {
            name: "",
            value: &self.property_data,
        }.encode_with_net(&mut buffer);

        buffer.put_u64(self.server_block_state_checksum);
        buffer.put_u128(self.world_template_id);
        buffer.put_bool(self.client_side_generation);

        Ok(buffer)
    }
}
