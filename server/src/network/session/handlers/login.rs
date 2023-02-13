use std::collections::HashMap;
use std::num::NonZeroU64;
use std::sync::atomic::Ordering;

use bytes::{BufMut, BytesMut};
use jsonwebtoken::jwk::KeyOperations::Encrypt;
use level::Dimension;

use crate::config::SERVER_CONFIG;
use crate::crypto::Encryptor;
use crate::network::packets::GameMode::Creative;
use crate::network::packets::{
    AvailableCommands, BiomeDefinitionList, BroadcastIntent,
    ChatRestrictionLevel, ChunkRadiusReply, ChunkRadiusRequest,
    ClientCacheStatus, ClientToServerHandshake, Command, CommandEnum,
    CommandOverload, CommandParameter, CommandParameterType,
    CommandPermissionLevel, CreativeContent, Difficulty, Disconnect,
    ExperimentData, GameMode, ItemEntry, Login, NetworkSettings,
    PermissionLevel, PlayStatus, PlayerMovementSettings, PlayerMovementType,
    RequestNetworkSettings, ResourcePackClientResponse, ResourcePackStack,
    ResourcePacksInfo, ServerToClientHandshake, SetLocalPlayerAsInitialized,
    SpawnBiomeType, StartGame, Status, ViolationWarning, WorldGenerator,
    DISCONNECTED_LOGIN_FAILED, DISCONNECTED_NOT_AUTHENTICATED, NETWORK_VERSION,
};
use crate::network::raknet::Reliability;
use crate::network::raknet::{Frame, FrameBatch};
use crate::network::session::send_queue::SendPriority;
use crate::network::session::session::Session;
use common::{bail, BlockPosition, Decodable, VResult, Vector2f, Vector3f};

impl Session {
    /// Handles a [`ClientCacheStatus`] packet.
    pub fn handle_client_cache_status(
        &self,
        mut packet: BytesMut,
    ) -> VResult<()> {
        let request = ClientCacheStatus::decode(packet)?;
        self.cache_support.set(request.supports_cache)?;

        Ok(())
    }

    pub fn handle_violation_warning(
        &self,
        mut packet: BytesMut,
    ) -> VResult<()> {
        let request = ViolationWarning::decode(packet)?;
        tracing::error!("Received violation warning: {request:?}");

        self.kick("Violation warning")?;
        Ok(())
    }

    pub fn handle_local_player_initialized(
        &self,
        mut packet: BytesMut,
    ) -> VResult<()> {
        let request = SetLocalPlayerAsInitialized::decode(packet)?;
        // Unused

        Ok(())
    }

    pub fn handle_chunk_radius_request(
        &self,
        mut packet: BytesMut,
    ) -> VResult<()> {
        let request = ChunkRadiusRequest::decode(packet)?;
        let reply = ChunkRadiusReply {
            allowed_radius: SERVER_CONFIG.read().allowed_render_distance,
        };
        self.send_packet(reply)?;

        Ok(())
    }

    pub fn handle_resource_pack_client_response(
        &self,
        mut packet: BytesMut,
    ) -> VResult<()> {
        let request = ResourcePackClientResponse::decode(packet)?;

        // TODO: Implement resource packs.

        let start_game = StartGame {
            entity_id: 1,
            runtime_id: 1,
            game_mode: GameMode::Creative,
            position: Vector3f::from([0.0, 0.0, 0.0]),
            rotation: Vector2f::from([90.0, 90.0]),
            world_seed: 69420,
            spawn_biome_type: SpawnBiomeType::Default,
            custom_biome_name: "plains".to_string(),
            dimension: Dimension::Overworld,
            generator: WorldGenerator::Infinite,
            world_game_mode: GameMode::Creative,
            difficulty: Difficulty::Normal,
            world_spawn: BlockPosition::new(0, 0, 0),
            achievements_disabled: true,
            editor_world: false,
            day_cycle_lock_time: 0,
            education_offer: 0,
            education_features_enabled: true,
            education_production_id: "".to_string(),
            rain_level: 0.0,
            lightning_level: 0.0,
            confirmed_platform_locked_content: false,
            broadcast_to_lan: true,
            xbox_broadcast_intent: BroadcastIntent::Public,
            platform_broadcast_intent: BroadcastIntent::Public,
            enable_commands: true,
            texture_packs_required: true,
            gamerules: vec![],
            experiments: vec![],
            experiments_previously_enabled: false,
            bonus_chest_enabled: false,
            starter_map_enabled: false,
            permission_level: PermissionLevel::Operator,
            server_chunk_tick_range: 0,
            has_locked_behavior_pack: false,
            has_locked_resource_pack: false,
            is_from_locked_world_template: false,
            use_msa_gamertags_only: false,
            is_from_world_template: false,
            is_world_template_option_locked: false,
            only_spawn_v1_villagers: false,
            persona_disabled: false,
            custom_skins_disabled: false,
            emote_chat_muted: true,
            limited_world_width: 0,
            limited_world_height: 0,
            force_experimental_gameplay: false,
            chat_restriction_level: ChatRestrictionLevel::None,
            disable_player_interactions: false,
            level_id: "".to_string(),
            level_name: "Nova Server".to_string(),
            template_content_identity: "".to_string(),
            is_trial: false,
            movement_settings: PlayerMovementSettings {
                movement_type: PlayerMovementType::ClientAuthoritative,
                rewind_history_size: 0,
                server_authoritative_breaking: true,
            },
            time: 0,
            enchantment_seed: 0,
            block_properties: vec![],
            item_properties: vec![],
            multiplayer_correlation_id: "".to_string(),
            server_authoritative_inventory: false,
            game_version: "1.19.60".to_string(),
            property_data: nbt::Value::Compound(HashMap::new()),
            server_block_state_checksum: 0,
            world_template_id: 0,
            client_side_generation: false,
        };
        self.send_packet(start_game)?;

        let creative_content = CreativeContent { items: vec![] };
        self.send_packet(creative_content)?;

        let biome_definition_list = BiomeDefinitionList;
        self.send_packet(biome_definition_list)?;

        let play_status = PlayStatus { status: Status::PlayerSpawn };
        self.send_packet(play_status)?;

        let available_commands = AvailableCommands {
            commands: vec![Command {
                name: "credits".to_owned(),
                description: "Shows the credits screen".to_owned(),
                permission_level: CommandPermissionLevel::Normal,
                aliases: vec![],
                overloads: vec![],
            }],
        };
        self.send_packet(available_commands)?;

        Ok(())
    }

    pub fn handle_client_to_server_handshake(
        &self,
        mut packet: BytesMut,
    ) -> VResult<()> {
        ClientToServerHandshake::decode(packet)?;

        let response = PlayStatus { status: Status::LoginSuccess };
        self.send_packet(response)?;

        // TODO: Implement resource packs
        let pack_info = ResourcePacksInfo {
            required: false,
            scripting_enabled: false,
            forcing_server_packs: false,
            behavior_info: vec![],
            resource_info: vec![],
        };
        self.send_packet(pack_info)?;

        let pack_stack = ResourcePackStack {
            forced_to_accept: false,
            resource_packs: vec![],
            behavior_packs: vec![],
            game_version: "1.19".to_string(),
            experiments: vec![],
            experiments_previously_toggled: false,
        };
        self.send_packet(pack_stack)?;

        Ok(())
    }

    /// Handles a [`Login`] packet.
    pub async fn handle_login(&self, mut packet: BytesMut) -> VResult<()> {
        let request = Login::decode(packet);
        let request = match request {
            Ok(r) => r,
            Err(e) => {
                self.kick(DISCONNECTED_LOGIN_FAILED)?;
                return Err(e);
            }
        };
        tracing::info!(
            "{} has joined the server",
            request.identity.display_name
        );

        let (encryptor, jwt) = Encryptor::new(&request.identity.public_key)?;

        self.identity.set(request.identity)?;
        self.user_data.set(request.user_data)?;

        // Flush packets before enabling encryption
        self.flush().await?;

        self.send_packet(ServerToClientHandshake { jwt: jwt.as_str() })?;
        self.encryptor.set(encryptor)?;

        Ok(())
    }

    /// Handles a [`RequestNetworkSettings`] packet.
    pub fn handle_request_network_settings(
        &self,
        mut packet: BytesMut,
    ) -> VResult<()> {
        let request = RequestNetworkSettings::decode(packet)?;
        if request.protocol_version != NETWORK_VERSION {
            if request.protocol_version > NETWORK_VERSION {
                let response = PlayStatus { status: Status::FailedServer };
                self.send_packet(response)?;

                bail!(
                    VersionMismatch,
                    "Client is using a newer protocol ({} vs. {})",
                    request.protocol_version,
                    NETWORK_VERSION
                );
            } else {
                let response = PlayStatus { status: Status::FailedClient };
                self.send_packet(response)?;

                bail!(
                    VersionMismatch,
                    "Client is using an older protocol ({} vs. {})",
                    request.protocol_version,
                    NETWORK_VERSION
                );
            }
        }

        let response = {
            let lock = SERVER_CONFIG.read();
            NetworkSettings {
                compression_algorithm: lock.compression_algorithm,
                compression_threshold: lock.compression_threshold,
                client_throttle: lock.client_throttle,
            }
        };

        self.send_packet(response)?;
        self.compression_enabled.store(true, Ordering::SeqCst);

        Ok(())
    }
}
