

use std::sync::atomic::Ordering;


use level::Dimension;

use crate::SERVER_CONFIG;
use crate::crypto::Encryptor;
use crate::CacheStatus;
use crate::AvailableCommands;
use crate::{
    BroadcastIntent, ChatRestrictionLevel, ChunkRadiusReply,
    ChunkRadiusRequest, ClientToServerHandshake, CreativeContent, Login, NetworkSettings, PermissionLevel, PlayerMovementSettings,
    PlayerMovementType, RequestNetworkSettings, ResourcePackClientResponse,
    ResourcePackStack, ResourcePacksInfo, ServerToClientHandshake,
    SpawnBiomeType, StartGame, WorldGenerator, DISCONNECTED_LOGIN_FAILED, Status, PlayStatus,
};

use crate::{
    BiomeDefinitionList, Difficulty, GameMode, MessageType,
    SetLocalPlayerAsInitialized, TextMessage,
    ViolationWarning, CLIENT_VERSION_STRING, NETWORK_VERSION,
};


use crate::Session;
use util::{
    bail, BlockPosition, Deserialize, Result, Vector2f, Vector3f,
};
use util::bytes::{MutableBuffer};

impl Session {
    /// Handles a [`ClientCacheStatus`] packet.
    /// This stores the result in the [`Session::cache_support`] field.
    pub fn process_cache_status(&self, pk: MutableBuffer) -> Result<()> {
        let request = CacheStatus::deserialize(pk.snapshot())?;
        self.cache_support.set(request.supports_cache)?;

        Ok(())
    }

    pub fn process_violation_warning(&self, pk: MutableBuffer) -> Result<()> {
        let request = ViolationWarning::deserialize(pk.snapshot())?;
        tracing::error!("Received violation warning: {request:?}");

        self.kick("Violation warning")?;
        Ok(())
    }

    /// Handles a [`SetLocalPlayerAsInitialized`] packet.
    /// This packet indicates the player has fully loaded in.
    ///
    /// All connected sessions are notified of the new player
    /// and the new player gets a list of all current players.
    pub fn process_local_initialized(&self, pk: MutableBuffer) -> Result<()> {
        let _request = SetLocalPlayerAsInitialized::deserialize(pk.snapshot())?;

        // Add player to other's player lists.
        tracing::info!("{} has connected", self.get_display_name()?);

        // Tell rest of server that this client has joined...
        {
            let identity_data = self.get_identity_data()?;
            let _user_data = self.get_user_data()?;

            // self.broadcast_others(PlayerListAdd {
            //     entries: &[PlayerListAddEntry {
            //         uuid: identity_data.uuid,
            //         entity_id: self.player.read().runtime_id as i64,
            //         username: &identity_data.display_name,
            //         xuid: identity_data.xuid,
            //         device_os: user_data.build_platform,
            //         skin: self.player.read().skin.as_ref().ok_or_else(
            //             || {
            //                 error!(
            //                     NotInitialized,
            //                     "Skin data has not been initialised"
            //                 )
            //             },
            //         )?,
            //         host: false,
            //     }],
            // })?;

            self.broadcast_others(TextMessage {
                message: &format!(
                    "§e{} has joined the server.",
                    identity_data.display_name
                ),
                needs_translation: false,
                parameters: vec![],
                source_name: "",
                platform_chat_id: "",
                message_type: MessageType::System,
                xuid: "",
            })?;
        }
        self.initialized.store(true, Ordering::SeqCst);

        // ...then tell the client about all the other players.
        // TODO

        Ok(())
    }

    /// Handles a [`ChunkRadiusRequest`] packet by returning the maximum allowed render distance.
    pub fn process_radius_request(&self, pk: MutableBuffer) -> Result<()> {
        let _request = ChunkRadiusRequest::deserialize(pk.snapshot())?;
        self.send(ChunkRadiusReply {
            allowed_radius: SERVER_CONFIG.read().allowed_render_distance,
        })
    }

    pub fn process_pack_client_response(
        &self,
        pk: MutableBuffer,
    ) -> Result<()> {
        let _request = ResourcePackClientResponse::deserialize(pk.snapshot())?;

        // TODO: Implement resource packs.

        let start_game = StartGame {
            entity_id: 1,
            runtime_id: 1,
            game_mode: self.get_game_mode(),
            position: Vector3f::from([0.0, 50.0, 0.0]),
            rotation: Vector2f::from([0.0, 0.0]),
            world_seed: 69420,
            spawn_biome_type: SpawnBiomeType::Default,
            custom_biome_name: "plains",
            dimension: Dimension::Overworld,
            generator: WorldGenerator::Infinite,
            world_game_mode: GameMode::Creative,
            difficulty: Difficulty::Normal,
            world_spawn: BlockPosition::new(0, 50, 0),
            achievements_disabled: true,
            editor_world: false,
            day_cycle_lock_time: 0,
            education_features_enabled: true,
            rain_level: 0.0,
            lightning_level: 0.0,
            confirmed_platform_locked_content: false,
            broadcast_to_lan: true,
            xbox_broadcast_intent: BroadcastIntent::Public,
            platform_broadcast_intent: BroadcastIntent::Public,
            enable_commands: true,
            texture_packs_required: true,
            game_rules: &self.level_manager.get_game_rules(),
            experiments: &[],
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
            level_id: "",
            level_name: "World name",
            template_content_identity: "",
            movement_settings: PlayerMovementSettings {
                movement_type: PlayerMovementType::ClientAuthoritative,
                rewind_history_size: 0,
                server_authoritative_breaking: true,
            },
            time: 0,
            enchantment_seed: 0,
            block_properties: &[],
            item_properties: &[],
            server_authoritative_inventory: false,
            game_version: "1.19.60",
            // property_data: nbt::Value::Compound(HashMap::new()),
            server_block_state_checksum: 0,
            world_template_id: 0,
            client_side_generation: false,
        };
        self.send(start_game)?;

        let creative_content = CreativeContent { items: &[] };
        self.send(creative_content)?;

        let biome_definition_list = BiomeDefinitionList;
        self.send(biome_definition_list)?;

        let play_status = PlayStatus { status: Status::PlayerSpawn };
        self.send(play_status)?;

        let commands = self
            .level_manager
            .get_commands()
            .iter()
            .map(|kv| kv.value().clone())
            .collect::<Vec<_>>();

        let available_commands =
            AvailableCommands { commands: commands.as_slice() };            

        self.send(available_commands)?;

        Ok(())
    }

    pub fn process_cts_handshake(&self, pk: MutableBuffer) -> Result<()> {
        ClientToServerHandshake::deserialize(pk.snapshot())?;

        let response = PlayStatus { status: Status::LoginSuccess };
        self.send(response)?;

        // TODO: Implement resource packs
        let pack_info = ResourcePacksInfo {
            required: false,
            scripting_enabled: false,
            forcing_server_packs: false,
            behavior_info: &[],
            resource_info: &[],
        };
        self.send(pack_info)?;

        let pack_stack = ResourcePackStack {
            forced_to_accept: false,
            resource_packs: &[],
            behavior_packs: &[],
            game_version: CLIENT_VERSION_STRING,
            experiments: &[],
            experiments_previously_toggled: false,
        };
        self.send(pack_stack)?;

        Ok(())
    }

    /// Handles a [`Login`] packet.
    pub async fn process_login(&self, pk: MutableBuffer) -> Result<()> {
        let request = Login::deserialize(pk.snapshot());
        let request = match request {
            Ok(r) => r,
            Err(e) => {
                self.kick(DISCONNECTED_LOGIN_FAILED)?;
                return Err(e);
            }
        };

        let (encryptor, jwt) = Encryptor::new(&request.identity.public_key)?;

        self.identity.set(request.identity)?;
        self.user_data.set(request.user_data)?;
        self.player.write().skin = Some(request.skin);

        // Flush packets before enabling encryption
        self.flush().await?;

        self.send(ServerToClientHandshake { jwt: &jwt })?;
        self.encryptor.set(encryptor)?;

        Ok(())
    }

    /// Handles a [`RequestNetworkSettings`] packet.
    pub fn process_network_settings_request(&self, pk: MutableBuffer) -> Result<()> {
        let request = RequestNetworkSettings::deserialize(pk.snapshot())?;
        if request.protocol_version != NETWORK_VERSION {
            if request.protocol_version > NETWORK_VERSION {
                let response = PlayStatus { status: Status::FailedServer };
                self.send(response)?;

                bail!(
                    Outdated,
                    "Client is using a newer protocol ({} vs. {})",
                    request.protocol_version,
                    NETWORK_VERSION
                );
            } else {
                let response = PlayStatus { status: Status::FailedClient };
                self.send(response)?;

                bail!(
                    Outdated,
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

        self.send(response)?;
        self.raknet
            .compression_enabled
            .store(true, Ordering::SeqCst);

        Ok(())
    }
}
