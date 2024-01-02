use uuid::Uuid;

use util::{Result, Serialize, Vector};
use util::{BinaryWrite, MutableBuffer};

use crate::bedrock::{AbilityData, DeviceOS, PermissionLevel};
use crate::bedrock::{ConnectedPacket, GameMode};
use crate::bedrock::command::CommandPermissionLevel;

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

impl EntityLink {
    pub fn encode(&self, buffer: &mut MutableBuffer) -> anyhow::Result<()> {
        buffer.write_var_i64(self.ridden_entity_id)?;
        buffer.write_var_i64(self.rider_entity_id)?;
        buffer.write_u8(self.link_type as u8)?;
        buffer.write_bool(self.is_immediate)?;
        buffer.write_bool(self.is_rider_initiated)
    }
}

/// Adds a player to the game.
/// A [`PlayerListAdd`](crate::bedrock::PlayerListAdd) packet, adding the player to the player list,
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
    // pub held_item: ItemStack,
    // pub metadata: HashMap<u32, nbt::Value>,
    // pub properties: EntityProperties,
    /// Abilities of the player. See [`AbilityData`].
    pub ability_data: AbilityData,
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
    fn serialize(&self, buffer: &mut MutableBuffer) -> anyhow::Result<()> {
        buffer.write_uuid_le(&self.uuid)?;
        buffer.write_str(self.username)?;
        buffer.write_var_u64(self.runtime_id)?;
        buffer.write_str("")?; // Platform chat ID
        buffer.write_vecf(&self.position)?;
        buffer.write_vecf(&self.velocity)?;
        buffer.write_vecf(&self.rotation)?;
        // self.held_item.serialize(buffer)?;
        buffer.write_var_i32(self.game_mode as i32)?;
        // buffer.put_metadata(&self.metadata);
        buffer.write_var_u32(0)?; // TODO: Entity metadata.
        buffer.write_var_u32(0)?; // Entity properties are unused.
        buffer.write_var_u32(0)?; // Entity properties are unused.
        self.ability_data.serialize(buffer)?;

        buffer.write_var_u32(self.links.len() as u32)?;
        for link in self.links {
            link.encode(buffer)?;
        }

        buffer.write_str(self.device_id)?;
        buffer.write_i32_le(self.device_os as i32)
    }
}