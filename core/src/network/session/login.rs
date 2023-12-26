use std::collections::HashMap;
use std::num::{NonZeroI32, NonZeroU32};
use std::sync::atomic::Ordering;

use level::Dimension;
use util::bytes::MutableBuffer;
use util::{bail, BlockPosition, Deserialize, Result, Vector};

use crate::config::SERVER_CONFIG;
use crate::crypto::Encryptor;
use crate::forms::{FormElement, FormInput, FormLabel, Modal};
use crate::network::{FormRequest, CacheStatus, HeightmapType, ItemCollection, NetworkChunkPublisherUpdate, SubChunkEntry, SubChunkResponse, SubChunkResult};
use crate::network::Session;
use crate::network::{AvailableCommands, SubChunkRequestMode};
use crate::network::{
    BiomeDefinitionList, Difficulty, GameMode, SetLocalPlayerAsInitialized, TextMessage, ViolationWarning, CLIENT_VERSION_STRING, NETWORK_VERSION,
};
use crate::network::{BlockEntry, ItemEntry, LevelChunk, PropertyData, TextData};
use crate::network::{
    BroadcastIntent, ChatRestrictionLevel, ChunkRadiusReply, ChunkRadiusRequest, ClientToServerHandshake, CreativeContent, Login, NetworkSettings,
    PermissionLevel, PlayStatus, PlayerMovementSettings, PlayerMovementType, RequestNetworkSettings, ResourcePackClientResponse, ResourcePackStack,
    ResourcePacksInfo, ServerToClientHandshake, SpawnBiomeType, StartGame, Status, WorldGenerator, DISCONNECTED_LOGIN_FAILED,
};

impl Session {
    /// Handles a [`CacheStatus`] packet.
    /// This stores the result in the [`Session::cache_support`] field.
    pub fn process_cache_status(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        let request = CacheStatus::deserialize(packet.snapshot())?;
        self.cache_support.set(request.supports_cache)?;

        tracing::debug!("[{}] Cache status is: {}", self.get_display_name()?, request.supports_cache);

        Ok(())
    }

    pub fn process_violation_warning(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        let request = ViolationWarning::deserialize(packet.snapshot())?;
        tracing::error!("Received violation warning: {request:?}");

        self.kick("Violation warning")?;
        Ok(())
    }

    /// Handles a [`SetLocalPlayerAsInitialized`] packet.
    /// This packet indicates the player has fully loaded in.
    ///
    /// All connected sessions are notified of the new player
    /// and the new player gets a list of all current players.
    pub fn process_local_initialized(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        let request = SetLocalPlayerAsInitialized::deserialize(packet.snapshot())?;
        tracing::debug!("[{}] Initialised with runtime ID {}", self.get_display_name()?, request.runtime_id);

        // Initialise chunk loading
        let lock = self.player.read();
        let rounded_position = Vector::from([
            lock.position.x.round() as i32,
            lock.position.y.round() as i32,
            lock.position.z.round() as i32
        ]);
        drop(lock);

        // Add player to other's player lists

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

            // let level_chunk = self.level_manager.request_biomes(Vector::from([0, 0]), Dimension::Overworld)?;
            // dbg!(level_chunk);

            self.broadcast(TextMessage {
                data: TextData::Translation {
                    parameters: vec![&format!("§e{}", identity_data.display_name)],
                    message: "multiplayer.player.joined"
                    // message: &format!("§e{} has joined the server.", identity_data.display_name),
                },
                needs_translation: true,
                xuid: "",
                platform_chat_id: "",
            })?;
        }
        self.initialized.store(true, Ordering::SeqCst);

        // ...then tell the client about all the other players.
        // TODO

        Ok(())
    }

    /// Handles a [`ChunkRadiusRequest`] packet by returning the maximum allowed render distance.
    pub fn process_radius_request(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        let request = ChunkRadiusRequest::deserialize(packet.snapshot())?;
        let allowed_radius = std::cmp::min(
            SERVER_CONFIG.read().allowed_render_distance, request.radius
        );

        self.send(ChunkRadiusReply {
            allowed_radius,
        })?;

        if request.radius <= 0 {
            anyhow::bail!("Render distance must be greater than 0");
        }
        tracing::debug!("Chunk radius updated to: {}", request.radius);

        {
            let player = self.player.read();
            player.viewer.set_radius(allowed_radius);
        }

        Ok(())
    }

    pub fn process_pack_client_response(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        tracing::debug!("Received client resource response");

        let _request = ResourcePackClientResponse::deserialize(packet.snapshot())?;

        // TODO: Implement resource packs.

        let start_game = StartGame {
            entity_id: 1,
            runtime_id: 1,
            game_mode: self.get_game_mode(),
            position: Vector::from([0.0, 60.0, 0.0]),
            rotation: Vector::from([0.0, 0.0]),
            world_seed: 0,
            spawn_biome_type: SpawnBiomeType::Default,
            custom_biome_name: "plains",
            dimension: Dimension::Overworld,
            generator: WorldGenerator::Infinite,
            world_game_mode: GameMode::Creative,
            difficulty: Difficulty::Normal,
            world_spawn: BlockPosition::new(0, 60, 0),
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
            game_rules: &self.level.get_game_rules(),
            experiments: &[],
            experiments_previously_enabled: false,
            bonus_chest_enabled: false,
            starter_map_enabled: true,
            permission_level: PermissionLevel::Operator,
            server_chunk_tick_range: 4,
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
                server_authoritative_breaking: false,
            },
            time: 0,
            enchantment_seed: 0,
            // block_properties: &[BlockEntry {
            //     name: "minecraft:bedrock".to_owned(),
            //     properties: HashMap::from([("infiniburn_bit".to_owned(), nbt::Value::Byte(0))]),
            // }],
            block_properties: &[],
            item_properties: &[],
            property_data: PropertyData {},
            server_authoritative_inventory: false,
            game_version: "1.20.50",
            // property_data: nbt::Value::Compound(HashMap::new()),
            server_block_state_checksum: 0,
            world_template_id: 0,
            client_side_generation: false,
            hashed_block_ids: false,
            server_authoritative_sounds: false
        };
        self.send(start_game)?;

        let creative_content = CreativeContent {
            items: &[]
        };
        self.send(creative_content)?;

        let biome_definition_list = BiomeDefinitionList;
        self.send(biome_definition_list)?;

        let commands = self.level.get_commands().iter().map(|kv| kv.value().clone()).collect::<Vec<_>>();
        let available_commands = AvailableCommands { commands: commands.as_slice() };
        self.send(available_commands)?;

        let play_status = PlayStatus { status: Status::PlayerSpawn };
        self.send(play_status)?;

        self.send(NetworkChunkPublisherUpdate {
            position: Vector::from([0, 0, 0]),
            radius: self.player
                .read()
                .viewer.get_radius() as u32
        })?;

        let subchunks = self.player.read().viewer.recenter(
            Vector::from([0, 0]), &(-4..16).map(|y| Vector::from([0, y, 0])).collect::<Vec<_>>()
        )?;
        let response = SubChunkResponse {
            entries: subchunks,
            position: Vector::from([0, 0, 0]),
            dimension: Dimension::Overworld,
            cache_enabled: false
        };
        self.send(response)?;

        Ok(())
    }

    pub fn process_cts_handshake(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        tracing::debug!("Handshake received");

        ClientToServerHandshake::deserialize(packet.snapshot())?;

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
    pub async fn process_login(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        let request = Login::deserialize(packet.snapshot());
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

        tracing::info!("`{}` has connected", self.get_display_name()?);

        Ok(())
    }

    /// Handles a [`RequestNetworkSettings`] packet.
    pub fn process_network_settings_request(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        let request = RequestNetworkSettings::deserialize(packet.snapshot())?;
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
        self.raknet.compression_enabled.store(true, Ordering::SeqCst);

        Ok(())
    }
}
