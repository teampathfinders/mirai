use uuid::Uuid;

use util::{Result, Serialize, Vector};
use util::bytes::{BinaryWrite, MutableBuffer};

use crate::network::{DeviceOS, PermissionLevel};
use crate::network::{ConnectedPacket, GameMode};
use crate::command::CommandPermissionLevel;
use crate::item::ItemStack;

/// Type of an entity link.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EntityLinkType {
    /// Removes the link between two entities.
    Remove,
    /// Link for entities that control what they're riding.
    Rider,
    /// Link for passengers, such as in a boat.
    Passenger,
}

/// Links multiple entities together.
/// This is used to make entities ride other entities, such as a player riding a horse.
#[derive(Debug, Clone)]
pub struct EntityLink {
    /// Type of the link.
    pub link_type: EntityLinkType,
    /// Entity unique ID of the ridden entity.
    pub ridden_entity_id: i64,
    /// Entity unique ID of the rider entity.
    pub rider_entity_id: i64,
    /// Whether to immediately unlink the entities, such as in the case of death of a horse.
    pub is_immediate: bool,
    /// Whether the link was initiated by the rider.
    pub is_rider_initiated: bool,
}

impl Serialize for EntityLink {
    fn serialize<W>(&self, writer: W) -> anyhow::Result<()>
    where
        W: BinaryWrite
    {
        writer.write_var_i64(self.ridden_entity_id)?;
        writer.write_var_i64(self.rider_entity_id)?;
        writer.write_u8(self.link_type as u8)?;
        writer.write_bool(self.is_immediate)?;
        writer.write_bool(self.is_rider_initiated)
    }
}

pub enum Ability {
    Build = 1 << 0,
    Mine = 1 << 1,
    DoorsAndSwitches = 1 << 2,
    OpenContainers = 1 << 3,
    AttackPlayers = 1 << 4,
    AttackMobs = 1 << 5,
    OperatorCommands = 1 << 6,
    Teleport = 1 << 7,
    Invulnerable = 1 << 8,
    Flying = 1 << 9,
    MayFly = 1 << 10,
    InstantBuild = 1 << 11,
    Lightning = 1 << 12,
    FlySpeed = 1 << 13,
    WalkSpeed = 1 << 14,
    Muted = 1 << 15,
    WorldBuilder = 1 << 16,
    NoClip = 1 << 17,
    Count = 1 << 18,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AbilityType {
    CustomCache,
    Base,
    Spectator,
    Commands,
    Editor,
}

#[derive(Debug, Clone)]
pub struct AbilityLayer {
    /// Type of ability layer.
    pub ability_type: AbilityType,
    /// Enabled abilities for this layer.
    pub abilities: u32,
    pub values: u32,
    /// Default fly speed.
    pub fly_speed: f32,
    /// Default walk speed.
    pub walk_speed: f32,
}

impl Serialize for AbilityLayer {
    fn serialize<W>(&self, writer: W) -> anyhow::Result<()>
    where
        W: BinaryWrite
    {
        writer.write_u16_le(self.ability_type as u16)?;
        writer.write_u32_le(self.abilities)?;
        writer.write_u32_le(self.values)?;
        writer.write_f32_le(self.fly_speed)?;
        writer.write_f32_le(self.walk_speed)
    }
}

#[derive(Debug, Clone)]
pub struct AbilityData<'a> {
    /// Entity unique ID.
    pub entity_id: i64,
    /// Player permission level (visitor, member, operator, etc.)
    /// This affects the icon shown in the player list.
    pub permission_level: PermissionLevel,
    /// The command permission level is separate from the standard level.
    /// This level affects which commands the player is allowed to execute.
    pub command_permission_level: CommandPermissionLevel,
    pub layers: &'a [AbilityLayer],
}

impl<'a> Serialize for AbilityData<'a> {
    fn serialize<W>(&self, writer: W) -> anyhow::Result<()>
    where
        W: BinaryWrite
    {
        writer.write_i64_le(self.entity_id)?; // For some reason this isn't a varint64.
        writer.write_u8(self.permission_level as u8)?;
        writer.write_u8(self.command_permission_level as u8)?;

        writer.write_u8(self.layers.len() as u8)?;
        for layer in self.layers {
            layer.serialize(writer)?;
        }

        Ok(())
    }
}

/// Adds a player to the game.
/// A [`PlayerListAdd`](crate::PlayerListAdd) packet, adding the player to the player list,
/// must be sent before using this.
#[derive(Debug, Clone)]
pub struct AddPlayer<'a> {
    /// UUID of the player to add to the game.
    pub uuid: Uuid,
    /// Username.
    pub username: &'a str,
    /// Runtime ID of the player.
    pub runtime_id: u64,
    /// Initial position.
    pub position: Vector<f32, 3>,
    /// Initial velocity.
    pub velocity: Vector<f32, 3>,
    /// Initial rotation.
    /// The third component is head yaw.
    pub rotation: Vector<f32, 3>,
    /// Game mode of the player.
    pub game_mode: GameMode,
    /// Item held by the player.
    pub held_item: ItemStack,
    // pub metadata: HashMap<u32, nbt::Value>,
    // pub properties: EntityProperties,
    /// Abilities of the player. See [`AbilityData`].
    pub ability_data: AbilityData<'a>,
    /// Entity links. See [`EntityLink`].
    pub links: &'a [EntityLink],
    /// ID of the user's device.
    pub device_id: &'a str,
    /// Device operating system.
    pub device_os: DeviceOS,
}

impl ConnectedPacket for AddPlayer<'_> {
    const ID: u32 = 0x0c;
}

impl Serialize for AddPlayer<'_> {
    fn serialize<W>(&self, writer: W) -> anyhow::Result<()>
    where
        W: BinaryWrite
    {
        writer.write_uuid_le(&self.uuid)?;
        writer.write_str(self.username)?;
        writer.write_var_u64(self.runtime_id)?;
        writer.write_str("")?; // Platform chat ID
        writer.write_vecf(&self.position)?;
        writer.write_vecf(&self.velocity)?;
        writer.write_vecf(&self.rotation)?;
        self.held_item.serialize(writer)?;
        writer.write_var_i32(self.game_mode as i32)?;
        // buffer.put_metadata(&self.metadata);
        writer.write_var_u32(0)?; // TODO: Entity metadata.
        writer.write_var_u32(0)?; // Entity properties are unused.
        writer.write_var_u32(0)?; // Entity properties are unused.
        self.ability_data.serialize(writer)?;

        writer.write_var_u32(self.links.len() as u32)?;
        for link in self.links {
            link.serialize(writer)?;
        }

        writer.write_str(self.device_id)?;
        writer.write_i32_le(self.device_os as i32)
    }
}