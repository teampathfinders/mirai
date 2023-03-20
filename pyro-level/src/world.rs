// Special keys

pub const AUTONOMOUS_ENTITIES: &[u8] = "AutonomousEntities".as_bytes();
pub const BIOME_DATA: &[u8] = "BiomeData".as_bytes();
pub const CHUNK_METADATA: &[u8] = "LevelChunkMetaDataDictionary".as_bytes();
pub const OVERWORLD: &[u8] = "Overworld".as_bytes();
pub const MOB_EVENTS: &[u8] = "mobevents".as_bytes();
pub const SCOREBOARD: &[u8] = "scoreboard".as_bytes();
pub const SCHEDULER: &[u8] = "schedulerWT".as_bytes();
pub const LOCAL_PLAYER: &[u8] = "~local_player".as_bytes();

#[derive(serde::Deserialize, Debug, PartialEq)]
pub struct Abilities {
    #[serde(rename = "attackmobs")]
    pub attack_mobs: bool,
    #[serde(rename = "attackplayers")]
    pub attack_players: bool,
    pub build: bool,
    #[serde(rename = "doorsandswitches")]
    pub doors_and_switches: bool,
    pub flying: bool,
    #[serde(rename = "instabuild")]
    pub instant_build: bool,
    pub invulnerable: bool,
    pub lightning: bool,
    pub mayfly: bool,
    pub mine: bool,
    pub op: bool,
    #[serde(rename = "opencontainers")]
    pub open_containers: bool,
    pub teleport: bool,
    #[serde(rename = "flySpeed")]
    pub fly_speed: f32,
    #[serde(rename = "walkSpeed")]
    pub walk_speed: f32,
}

#[derive(serde::Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Experiments {
    pub experiments_ever_used: bool,
    pub saved_with_toggled_experiments: bool,
}

#[derive(serde::Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Policies {
    // Not sure what is supposed to be in here
}

#[derive(serde::Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct LevelData {
    pub lightning_level: f32,
    pub lightning_time: i32,
    pub rain_level: f32,
    pub rain_time: i32,
    #[serde(rename = "Difficulty")]
    pub difficulty: i32,
    #[serde(rename = "GameType")]
    pub game_mode: i32,
    #[serde(rename = "Generator")]
    pub generator: i32,
    #[serde(rename = "LimitedWorldOriginX")]
    pub limited_world_origin_x: i32,
    #[serde(rename = "LimitedWorldOriginY")]
    pub limited_world_origin_y: i32,
    #[serde(rename = "LimitedWorldOriginZ")]
    pub limited_world_origin_z: i32,
    pub limited_world_depth: i32,
    pub limited_world_width: i32,
    #[serde(rename = "MinimumCompatibleClientVersion")]
    pub minimum_compatible_client_version: [i32; 5],
    // pub minimum_compatible_client_version: f32,
    #[serde(rename = "NetherScale")]
    pub nether_scale: i32,
    #[serde(rename = "NetworkVersion")]
    pub network_version: i32,
    #[serde(rename = "Platform")]
    pub platform: i32,
    #[serde(rename = "PlatformBroadcastIntent")]
    pub platform_broadcast_intent: i32,
    #[serde(rename = "RandomSeed")]
    pub random_seed: i64,
    #[serde(rename = "SpawnV1Villagers")]
    pub spawn_v1_villagers: bool,
    #[serde(rename = "SpawnX")]
    pub spawn_x: i32,
    #[serde(rename = "SpawnY")]
    pub spawn_y: i32,
    #[serde(rename = "SpawnZ")]
    pub spawn_z: i32,
    #[serde(rename = "StorageVersion")]
    pub storage_version: i32,
    #[serde(rename = "Time")]
    pub time: i64,
    #[serde(rename = "WorldVersion")]
    pub world_version: i32,
    #[serde(rename = "XBLBroadcastIntent")]
    pub xbox_broadcast_intent: i32,
    pub current_tick: i64,
    pub experiments: Experiments,
    pub abilities: Abilities,
    pub edu_offer: i32,
    pub education_features_enabled: bool,
    #[serde(rename = "lastOpenedWithVersion")]
    pub last_opened_with_version: [i32; 5],
    pub bonus_chest_enabled: bool,
    pub bonus_chest_spawned: bool,
    #[serde(rename = "commandblockoutput")]
    pub command_block_output: bool,
    #[serde(rename = "CenterMapsToOrigin")]
    pub center_maps_to_origin: bool,
    #[serde(rename = "commandblocksenabled")]
    pub command_blocks_enabled: bool,
    pub commands_enabled: bool,
    #[serde(rename = "ConfirmedPlatformLockedContent")]
    pub confirmed_platform_locked_content: bool,
    #[serde(rename = "dodaylightcycle")]
    pub daylight_cycle: bool,
    #[serde(rename = "doentitydrops")]
    pub entity_drops: bool,
    #[serde(rename = "dofiretick")]
    pub fire_tick: bool,
    #[serde(rename = "doimmediaterespawn")]
    pub immediate_respawn: bool,
    #[serde(rename = "doinsomnia")]
    pub insomnia: bool,
    #[serde(rename = "domobloot")]
    pub mob_loot: bool,
    #[serde(rename = "domobspawning")]
    pub mob_spawning: bool,
    #[serde(rename = "dotiledrops")]
    pub tile_drops: bool,
    #[serde(rename = "doweathercycle")]
    pub weather_cycle: bool,
    #[serde(rename = "drowningdamage")]
    pub drowning_damage: bool,
    #[serde(rename = "falldamage")]
    pub fall_damage: bool,
    #[serde(rename = "firedamage")]
    pub fire_damage: bool,
    #[serde(rename = "freezedamage")]
    pub freeze_damage: bool,
    #[serde(rename = "keepinventory")]
    pub keep_inventory: bool,
    #[serde(rename = "maxcommandchainlength")]
    pub max_command_chain_length: i32,
    #[serde(rename = "mobgriefing")]
    pub mob_griefing: bool,
    #[serde(rename = "naturalregeneration")]
    pub natural_regeneration: bool,
    #[serde(rename = "functioncommandlimit")]
    pub function_command_limit: i32,
    pub pvp: bool,
    #[serde(rename = "randomtickspeed")]
    pub random_tick_speed: i32,
    #[serde(rename = "respawnblocksexplode")]
    pub respawn_blocks_explode: bool,
    #[serde(rename = "sendcommandfeedback")]
    pub send_command_feedback: bool,
    #[serde(rename = "showbordereffect")]
    pub show_border_effect: bool,
    #[serde(rename = "showcoordinates")]
    pub show_coordinates: bool,
    #[serde(rename = "showdeathmessages")]
    pub show_death_messages: bool,
    #[serde(rename = "showtags")]
    pub show_tags: bool,
    #[serde(rename = "spawnradius")]
    pub spawn_radius: i32,
    #[serde(rename = "tntexplodes")]
    pub tnt_explodes: bool,
    #[serde(rename = "ForceGameType")]
    pub force_game_mode: bool,
    pub has_been_loaded_in_creative: bool,
    pub has_locked_behavior_pack: bool,
    pub has_locked_resource_pack: bool,
    pub immutable_world: bool,
    pub is_from_locked_template: bool,
    pub is_from_world_template: bool,
    pub is_single_use_world: bool,
    pub is_world_template_option_locked: bool,
    pub requires_copied_pack_removal_check: bool,
    pub texture_packs_required: bool,
    #[serde(rename = "LANBroadcast")]
    pub lan_broadcast: bool,
    #[serde(rename = "LANBroadcastIntent")]
    pub lan_broadcast_intent: i8,
    #[serde(rename = "MultiplayerGame")]
    pub multiplayer_game: bool,
    #[serde(rename = "MultiplayerGameIntent")]
    pub multiplayer_game_intent: i8,
    #[serde(rename = "LastPlayed")]
    pub last_played: i64,
    pub base_game_version: String,
    #[serde(rename = "BiomeOverride")]
    pub biome_override: String,
    #[serde(rename = "FlatWorldLayers")]
    pub flat_world_layers: String,
    #[serde(rename = "InventoryVersion")]
    pub inventory_version: String,
    #[serde(rename = "LevelName")]
    pub level_name: String,
    pub use_msa_gamertags_only: bool,
    pub world_start_count: i64,
    pub start_with_map_enabled: bool,
    pub spawn_mobs: bool,
    pub server_chunk_tick_range: i32,
    pub permissions_level: i32,
    pub player_permissions_level: i32,
    pub prid: String,
    #[serde(rename = "world_policies")]
    pub world_policies: Policies,
}

/// Database key prefixes.
///
/// Data from [`Minecraft fandom`](https://minecraft.fandom.com/wiki/Bedrock_Edition_level_format#Chunk_key_format).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum KeyData {
    /// 3D biome map.
    Biome3d = 0x2b,
    /// Version of the specified chunk.
    ChunkVersion = 0x2c,
    HeightMap = 0x2d,
    /// Sub chunk data.
    SubChunk {
        index: i8,
    } = 0x2f,
    /// A block entity.
    BlockEntity = 0x31,
    /// An entity.
    Entity = 0x32,
    /// Pending tick data.
    PendingTicks = 0x33,
    /// Biome state.
    BiomeState = 0x35,
    /// Finalized state.
    FinalizedState = 0x36,
    /// Education Edition border blocks.
    BorderBlocks = 0x38,
    /// Bounding boxes for structure spawns stored in binary format.
    HardCodedSpawnAreas = 0x39,
    /// Random tick data.
    RandomTicks = 0x3a,
}

impl KeyData {
    pub fn discriminant(&self) -> u8 {
        // SAFETY: KeyData is marked as `repr(u8)` and therefore its layout is a
        // `repr(C)` union of `repr(C)` structs, each of which has the `u8` discriminant as its first
        // field. Hence, we can read the discriminant without offsetting the pointer.
        unsafe { *<*const _>::from(self).cast::<u8>() }
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseKey {
    /// X coordinate of the chunk.
    pub x: i32,
    /// Z coordinate of the chunk.
    pub z: i32,
    /// Dimension of the chunk.
    pub dimension: Dimension,
    /// The tag of the data to load.
    pub data: KeyData,
}

impl DatabaseKey {
    pub(crate) fn serialized_size(&self) -> usize {
        4 + 4
            + if self.dimension != Dimension::Overworld {
            4
        } else {
            0
        }
            + 1
            + if let KeyData::SubChunk { .. } = self.data {
            1
        } else {
            0
        }
    }

    // pub(crate) fn serialize(&self, buffer: &mut MutableBuffer) -> Result<()> {
    //     buffer.write_i32_le(self.x);
    //     buffer.write_i32_le(self.z);
    //
    //     if self.dimension != Dimension::Overworld {
    //         buffer.write_i32_le(self.dimension as i32);
    //     }
    //
    //     buffer.write_u8(self.data.discriminant());
    //     if let KeyData::SubChunk { index } = self.data {
    //         buffer.write_le::<i8>(index);
    //     }
    // }
}

/// The Minecraft dimensions.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Dimension {
    /// The overworld dimension.
    Overworld,
    /// The nether dimension.
    Nether,
    /// The end dimension.
    End,
}
