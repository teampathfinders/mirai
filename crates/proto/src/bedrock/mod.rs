//! All types and packets implemented in the Bedrock protocol.

use util::glob_export;

glob_export!(login);
glob_export!(cache);
glob_export!(command);
glob_export!(input);
glob_export!(level);
glob_export!(settings);

glob_export!(action);
glob_export!(add_player);
glob_export!(add_painting);
glob_export!(animate);
glob_export!(available_actor_identifiers);
glob_export!(biome_definition_list);
glob_export!(block_event);
glob_export!(block_pick_request);
glob_export!(book_edit);
glob_export!(boss_event);
glob_export!(camera_shake);
glob_export!(change_dimension);
glob_export!(client_bound_debug_renderer);
glob_export!(container_close);
glob_export!(container_open);
glob_export!(death_info);
glob_export!(event);
glob_export!(form_request);
glob_export!(form_response);
glob_export!(generic_level_event);
glob_export!(header);
glob_export!(interact);
glob_export!(inventory_options);
glob_export!(level_event);
glob_export!(mob_effect);
glob_export!(network_chunk_publisher_update);
glob_export!(play_sound);
glob_export!(player_list);
glob_export!(request_ability);
glob_export!(respawn);
glob_export!(set_hud);
glob_export!(set_local_player_as_initialized);
glob_export!(show_credits);
glob_export!(show_profile);
glob_export!(simple_event);
glob_export!(skin);
glob_export!(spawn_experience_orb);
glob_export!(text);
glob_export!(tick_sync);
glob_export!(toast_request);
glob_export!(traits);
glob_export!(transfer);
glob_export!(update_abilities);
glob_export!(update_dynamic_enum);
glob_export!(update_fog_stack);
glob_export!(violation_warning);

/// ID of Minecraft game raknet.
pub const CONNECTED_PACKET_ID: u8 = 0xfe;
/// Semver version that this server supports.
pub const CLIENT_VERSION_STRING: &str = "1.21";
/// Protocol version that this server supports.
pub const PROTOCOL_VERSION: u32 = 686;
