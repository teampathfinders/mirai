#[derive(Debug)]
pub struct StartGame {
    pub entity_id_self	: i64,
    pub runtime_id_self	: u64,
    pub player_gamemode	: i32,
    pub spawn	: i32,//
    // vector 3^
    pub rotation	: i32,//
    // Vector2^
    pub seed	: i32,
    pub spawn_biome_type	: i16,
    pub custom_biome_name	: String,
    pub dimension	: i32,
    pub generator	: i32,
    pub world_gamemode	: i32,
    pub difficulty	: i32,
    pub world_spawn	: i32,
    // ^BlockCoordinates
    pub has_achievements_disabled	: Bool,
    pub day_cycle_stop_time	: i32,
    pub edu_offer:i32,
    pub has_education_edition_features_enabled:i32,
    pub education_production_id:bool,
    pub rain_level:String,
    pub lightning_level:f32,
    pub has_confirmed_platform_locked_content:f32,
    pub is_multiplayer:bool,
    pub broadcast_to_lan:bool,
    pub xbox_live_broadcast_mode:bool,
    pub platform_broadcast_mode:i32,
    pub enable_commands:i32,
    pub are_texture_packs_required:bool,
    pub gamerules:bool,
    pub bonus_chest:String,
    // ^GameRules
    pub map_enabled:bool,
    pub permission_level:bool,
    pub server_chunk_tick_range:i32,
    pub has_locked_behavior_pack:i32,
    pub has_locked_resource_pack:bool,
    pub is_from_locked_world_template:bool,
    pub use_msa_gamertags_only:bool,
    pub is_from_world_template:bool,
    pub is_world_template_option_locked:bool,
    pub only_spawn_v1_villagers:bool,
    pub game_version:bool,
    pub limited_world_width:String,
    pub limited_world_height:i32,
    pub is_nether_type:i32,
    pub is_force_experimental_gameplay:bool,
    pub level_id:bool,
    pub world_name:String,
    pub premium_world_template_id:String,
    pub is_trial:String,
    pub movement_type:bool,
    pub movement_rewind_size:u32,
    pub server_authoritative_block_breaking:i32,
    pub current_tick:bool,
    pub enchantment_seed:u64,
    pub block_properties:i32,
    // block_properties
    pub itemstates:i32,
    // ^Itemstates
    pub multiplayer_correlation_id:String,
    pub inventories_server_authoritative:bool,







}


impl StartGame {
    pub const ID: u8 = 0x0B;
}
