use std::collections::HashMap;
use uuid::Uuid;
use crate::types::Dimension;
use macros::variant_count;
use util::{Serialize, Vector};

use util::BlockPosition;
use util::{BinaryWrite, VarInt, VarString};

use crate::bedrock::{CLIENT_VERSION_STRING, ConnectedPacket, Difficulty, GameMode, GameRule, GameRulesChanged, ExperimentList};
use crate::bedrock::Experiment;

const MULTIPLAYER_CORRELATION_ID: &str = "5b39a9d6-f1a1-411a-b749-b30742f81771";

/// Which world generator type the server is using.
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
    /// Serializes the enum.
    pub fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_var_i32(*self as i32)
    }
}

/// The permission level of the client.
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
#[variant_count]
pub enum PermissionLevel {
    /// A visitor is a player that has most permissions removed.
    Visitor,
    /// A member is the default permission level for players.
    Member,
    /// An operator is a player that is allowed to run commands.
    Operator,
    /// A combination of any of the above.
    Custom,
}

impl TryFrom<u8> for PermissionLevel {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> anyhow::Result<PermissionLevel> {
        if value <= PermissionLevel::variant_count() as u8 {
            // SAFETY: This is safe because the discriminant is in range and
            // the representations are the same. Additionally, none of the enum members
            // have a manually assigned value (this is ensured by the `variant_count` macro).
            Ok(unsafe {
                std::mem::transmute::<u8, PermissionLevel>(value)
            })
        } else {
            anyhow::bail!("Permission level out of range <= 3, got {value}")
        }
    }
}

#[derive(Debug, Clone)]
pub struct EducationResourceURI {
    pub button_name: String,
    pub link_uri: String,
}

impl EducationResourceURI {
    /// Serializes the struct.
    pub fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_str(&self.button_name)?;
        writer.write_str(&self.link_uri)
    }
}

/// How restricted chat is.
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum ChatRestrictionLevel {
    /// Chat has no restrictions, this is how Minecraft normally works.
    None,
    /// Same as `Disabled`.
    Dropped,
    /// Prevents the user from sending any chat messages and displays `Chat is disabled`.
    Disabled,
}

impl ChatRestrictionLevel {
    /// Serializes the enum.
    pub fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_u8(*self as u8)
    }
}

/// Determines how player movement is handled.
#[derive(Debug, Copy, Clone)]
#[repr(i32)]
pub enum PlayerMovementType {
    /// The client has full control over its movement.
    ClientAuthoritative,
    /// The server needs to authorise player movement.
    ServerAuthoritative,
    /// Same as `ServerAuthoritative` but adds the ability to rewind player movement.
    ServerAuthoritativeWithRewind,
}

/// Sets the player movement settings.
#[derive(Debug, Copy, Clone)]
pub struct PlayerMovementSettings {
    // /// See [`PlayerMovementType`].
    // pub movement_type: PlayerMovementType,
    /// Determines how far back a rewind can go.
    pub rewind_history_size: i32,
    /// Whether the server authorises block breaking.
    pub server_authoritative_breaking: bool,
}

impl PlayerMovementSettings {
    /// Returns the serialized size of the struct.
    pub fn serialized_size(&self) -> usize {
        // (self.movement_type as i32).var_len() +
            self.rewind_history_size.var_len() +
            1
    }

    /// Serializes the struct.
    pub fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        // writer.write_var_i32(self.movement_type as i32)?;
        writer.write_var_i32(self.rewind_history_size)?;
        writer.write_bool(self.server_authoritative_breaking)
    }
}

#[derive(Debug)]
pub struct BlockEntry {
    /// Name of the block.
    pub name: String,
    // NBT compound containing properties.
    pub properties: HashMap<String, nbt::Value>,
}

impl BlockEntry {
    pub const fn serialized_size(&self) -> usize {
        1
        // self.name.var_len() //+ self.properties.serialized_net_size("")
    }
}

impl Serialize for BlockEntry {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        // {
        //     let mut buf2 = Vec::new();
        //     buf2.write_str(&self.name)?;
        //     nbt::to_var_bytes_in(&mut buf2, &self.properties)?;

        //     let mut ss: &mut &mut [u8] = &mut buf2.as_mut();
        //     let name = ss.read_str()?;
        //     let (properties, _): (nbt::Value, usize) = nbt::from_var_bytes(*ss)?;

        //     dbg!(name, properties);
        // }

        writer.write_str(&self.name)?;
        nbt::to_var_bytes_in(writer, &self.properties)
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

impl Serialize for ItemEntry {
    fn size_hint(&self) -> Option<usize> {
        Some(self.name.var_len() + 2 + 1)
    }

    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_str(&self.name)?;
        writer.write_u16_le(self.runtime_id)?;
        writer.write_bool(self.component_based)
    }
}

#[derive(Debug, Copy, Clone, serde::Serialize)]
#[serde(rename = "")]
pub struct PropertyData {}

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

impl Serialize for BroadcastIntent {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_var_u32(*self as u32)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(i32)]
pub enum EditorWorldType {
    NotEditor,
    Project,
    TestLevel
}

impl Serialize for EditorWorldType {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_var_i32(*self as i32)
    }
}

#[derive(Debug)]
pub struct SpawnSettings<'a> {
    pub ty: u16,
    pub biome_name: &'a str,
    pub dimension: Dimension
}

impl<'a> Serialize for SpawnSettings<'a> {
    fn serialize_into<W: BinaryWrite>(&self, mut writer: &mut W) -> anyhow::Result<()> {
        writer.write_u16_le(self.ty)?;
        writer.write_str(self.biome_name)?;
        writer.write_var_i32(self.dimension as i32)?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct LevelSettings<'a> {
    pub seed: u64,
    pub spawn_settings: SpawnSettings<'a>,
    pub generator_type: u32,
    pub game_type: GameMode,
    pub hardcore: bool,
    pub difficulty: Difficulty,
    pub default_spawn: BlockPosition,
    pub achievements_disabled: bool,
    pub editor_world_type: EditorWorldType,
    pub created_in_editor: bool,
    pub exported_from_editor: bool,
    pub day_cycle_stop_time: u32,
    pub edu_offer: u32,
    pub edu_enabled: bool,
    pub edu_product_id: &'a str,
    pub rain_level: f32,
    pub lightning_level: f32,
    pub platform_locked_content: bool,
    pub multiplayed_enabled: bool,
    pub lan_broadcasting: bool,
    pub xbox_live_broadcast: BroadcastIntent,
    pub platform_broadcast: BroadcastIntent,
    pub commands_enabled: bool,
    pub texture_packs_required: bool,
    pub rule_data: GameRulesChanged<'a>,
    pub experiments: ExperimentList<'a>,
    pub bonus_chest: bool,
    pub starter_map: bool,
    pub permission_level: PermissionLevel,
    pub server_tick_range: i32,
    pub behavior_pack_locked: bool,
    pub resource_pack_locked: bool,
    pub from_locked_template: bool,
    pub msa_gamertags_only: bool,
    pub from_template: bool,
    pub template_locked_settings: bool,
    pub only_v1_villagers: bool,
    pub persona_disabled: bool,
    pub custom_skins_disabled: bool,
    pub emote_chat_muted: bool,
    pub base_game_version: &'a str,
    pub world_width: i32,
    pub world_depth: i32,
    pub nether_type: bool,
    pub edu_resource_uri: EducationResourceURI,
    pub force_experimental_gameplay: bool,
    pub chat_restriction_level: ChatRestrictionLevel,
    pub disable_player_interactions: bool,
    pub server_identifier: &'a str,
    pub world_identifier: &'a str,
    pub scenario_identifier: &'a str,
    pub owner_identifier: &'a str
}

impl<'a> Serialize for LevelSettings<'a> {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_u64_le(self.seed)?;
        self.spawn_settings.serialize_into(writer)?;
        writer.write_var_i32(self.generator_type as i32)?;
        writer.write_var_i32(self.game_type as i32)?;
        writer.write_bool(self.hardcore)?;
        writer.write_var_i32(self.difficulty as i32)?;
        writer.write_block_pos(&self.default_spawn)?;
        writer.write_bool(self.achievements_disabled)?;
        writer.write_var_i32(self.editor_world_type as i32)?;
        writer.write_bool(self.created_in_editor)?;
        writer.write_bool(self.exported_from_editor)?;
        writer.write_var_i32(self.day_cycle_stop_time as i32)?;
        writer.write_var_i32(self.edu_offer as i32)?;
        writer.write_bool(self.edu_enabled)?;
        writer.write_str(&self.edu_product_id)?;
        writer.write_f32_le(self.rain_level)?;
        writer.write_f32_le(self.lightning_level)?;
        writer.write_bool(self.platform_locked_content)?;
        writer.write_bool(self.multiplayed_enabled)?;
        writer.write_bool(self.lan_broadcasting)?;
        writer.write_var_i32(self.xbox_live_broadcast as i32)?;
        writer.write_var_i32(self.platform_broadcast as i32)?;
        writer.write_bool(self.commands_enabled)?;
        writer.write_bool(self.texture_packs_required)?;
        self.rule_data.serialize_into(writer)?;
        self.experiments.serialize_into(writer)?;
        writer.write_bool(self.bonus_chest)?;
        writer.write_bool(self.starter_map)?;
        writer.write_var_i32(self.permission_level as i32)?;
        writer.write_i32_le(self.server_tick_range)?;
        writer.write_bool(self.behavior_pack_locked)?;
        writer.write_bool(self.resource_pack_locked)?;
        writer.write_bool(self.from_locked_template)?;
        writer.write_bool(self.msa_gamertags_only)?;
        writer.write_bool(self.from_template)?;
        writer.write_bool(self.template_locked_settings)?;
        writer.write_bool(self.only_v1_villagers)?;
        writer.write_bool(self.persona_disabled)?;
        writer.write_bool(self.custom_skins_disabled)?;
        writer.write_bool(self.emote_chat_muted)?;
        writer.write_str(&self.base_game_version)?;
        writer.write_i32_le(self.world_width)?;
        writer.write_i32_le(self.world_depth)?;
        writer.write_bool(self.nether_type)?;
        self.edu_resource_uri.serialize_into(writer)?;
        writer.write_bool(self.force_experimental_gameplay)?; // TODO
        writer.write_u8(self.chat_restriction_level as u8)?;
        writer.write_bool(self.disable_player_interactions)?;
        writer.write_str(&self.server_identifier)?;
        writer.write_str(&self.world_identifier)?;
        writer.write_str(&self.scenario_identifier)?;
        writer.write_str(&self.owner_identifier)?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct NetworkPermissions {
    pub server_authoritative_sound: bool
}

impl Serialize for NetworkPermissions {
    fn serialize_into<W: BinaryWrite>(&self, mut writer: &mut W) -> anyhow::Result<()> {
        writer.write_bool(self.server_authoritative_sound)
    }
}

/// The start game packet contains most of the world settings displayed in the settings menu.
#[derive(Debug)]
pub struct StartGame<'a> {
    pub entity_id: i64,
    /// Runtime ID of the client.
    pub runtime_id: u64,
    /// Current game mode of the client.
    /// This is not the same as the world game mode.
    pub game_mode: GameMode,
    /// Spawn position.
    pub position: Vector<f32, 3>,
    /// Spawn rotation.
    pub rotation: Vector<f32, 2>,
    pub level_settings: LevelSettings<'a>,

    pub level_id: &'a str,
    /// Name of the world.
    /// This is shown in the pause menu above the player list, and the settings menu.
    pub level_name: &'a str,
    pub template_content_identity: &'a str,
    pub movement_settings: PlayerMovementSettings,
    /// Current time.
    pub time: u64,
    pub enchantment_seed: i32,
    pub block_properties: &'a [BlockEntry],
    pub item_properties: &'a [ItemEntry],
    pub property_data: PropertyData,
    /// Whether inventory transactions are server authoritative.
    pub server_authoritative_inventory: bool,
    /// Version of the game that the server is running.
    pub game_version: &'a str,
    // pub property_data: nbt::Value,
    pub server_block_state_checksum: u64,
    pub world_template_id: u128,
    /// Client side generation allows the client to generate its own chunks without the server having to send them over.
    pub client_side_generation: bool,
    pub hashed_block_ids: bool,
    pub server_version: &'a str,
    pub network_permissions: NetworkPermissions,
    pub tick_death_systems_enabled: bool
}

impl ConnectedPacket for StartGame<'_> {
    const ID: u32 = 0x0b;

    fn serialized_size(&self) -> usize {
        // TODO: Update to new version
        self.entity_id.var_len() +
            self.runtime_id.var_len() +
            (self.game_mode as i32).var_len() +
            3 * 4 +
            3 * 4 +
            8 +
            2 +
            // self.custom_biome_name.var_len() +
            // (self.dimension as u32).var_len() +
            // (self.generator as i32).var_len() +
            // (self.world_game_mode as i32).var_len() +
            // (self.difficulty as i32).var_len() +
            1 +
            // self.world_spawn.serialized_size() +
            1 +
            1 +
            // self.day_cycle_lock_time.var_len() +
            0.var_len() +
            1 +
            "".var_len() +
            4 +
            4 +
            1 +
            1 +
            1 +
            // (self.xbox_broadcast_intent as u32).var_len() +
            // (self.platform_broadcast_intent as u32).var_len() +
            1 +
            1 +
            // (self.game_rules.len() as u32).var_len() +
            // self.game_rules.iter().fold(0, |acc, r| acc + r.serialized_size()) +
            4 +
            // self.experiments.iter().fold(0, |acc, e| acc + e.serialized_size()) +
            1 +
            1 +
            1 +
            // (self.permission_level as i32).var_len() +
            4 +
            1 +
            1 +
            1 +
            1 +
            1 +
            1 +
            1 +
            1 +
            1 +
            1 +
            CLIENT_VERSION_STRING.var_len() +
            4 +
            4 +
            1 +
            "".var_len() +
            "".var_len() +
            1 +
            1 +
            1 +
            self.level_id.var_len() +
            self.level_name.var_len() +
            self.template_content_identity.var_len() +
            1 +
            self.movement_settings.serialized_size() +
            8 +
            self.enchantment_seed.var_len() +
            (self.block_properties.len() as u32).var_len() +
            self.block_properties.iter().fold(0, |acc, p| acc + p.size_hint().unwrap()) +
            (self.item_properties.len() as u32).var_len() +
            self.item_properties.iter().fold(0, |acc, p| acc + p.size_hint().unwrap()) +
            MULTIPLAYER_CORRELATION_ID.var_len() +
            1 +
            CLIENT_VERSION_STRING.var_len() +
            // self.property_data.serialized_net_size("") +
            8 +
            16 +
            1 +
            1 +
            1 +
            1 +
            1 + 3 + 3
    }
}

impl<'a> Serialize for StartGame<'a> {
    fn serialize_into<W: BinaryWrite>(&self, mut writer: &mut W) -> anyhow::Result<()> {
        writer.write_var_i64(self.entity_id)?;
        writer.write_var_u64(self.runtime_id)?;

        writer.write_var_i32(self.game_mode as i32)?;
        writer.write_vecf(&self.position)?;
        writer.write_vecf(&self.rotation)?;

        self.level_settings.serialize_into(writer)?;

        writer.write_str(self.level_id)?;
        writer.write_str(self.level_name)?;
        writer.write_str(self.template_content_identity)?;
        writer.write_bool(false)?; // Is trial
        self.movement_settings.serialize_into(writer)?;

        writer.write_u64_le(self.time)?;
        writer.write_var_i32(self.enchantment_seed)?;

        // Block properties
        writer.write_var_u32(self.block_properties.len() as u32)?;

        writer.write_str("")?; // Multiplayer correlation ID
        writer.write_bool(true)?; // Enable item stack net manager (server authoritative inventory)
        writer.write_str(self.server_version)?;
        nbt::to_var_bytes_in(&mut writer, &self.property_data)?;
        writer.write_u64_le(self.server_block_state_checksum)?;
        writer.write_uuid_le(&Uuid::new_v4())?; // World template ID
        writer.write_bool(self.client_side_generation)?;
        writer.write_bool(self.hashed_block_ids)?;
        writer.write_bool(self.tick_death_systems_enabled)?;
        self.network_permissions.serialize_into(writer)?;

        Ok(())
    }
}
