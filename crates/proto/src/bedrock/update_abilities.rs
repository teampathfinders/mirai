use macros::variant_count;
use util::{BinaryRead, BinaryWrite};
use util::{Deserialize, Serialize};
use crate::bedrock::command::CommandPermissionLevel;
use crate::bedrock::{ConnectedPacket, PermissionLevel};

// #[derive(Debug)]
// pub enum Ability {
//     Build = 1 << 0,
//     Mine = 1 << 1,
//     DoorsAndSwitches = 1 << 2,
//     OpenContainers = 1 << 3,
//     AttackPlayers = 1 << 4,
//     AttackMobs = 1 << 5,
//     OperatorCommands = 1 << 6,
//     Teleport = 1 << 7,
//     Invulnerable = 1 << 8,
//     Flying = 1 << 9,
//     MayFly = 1 << 10,
//     InstantBuild = 1 << 11,
//     Lightning = 1 << 12,
//     FlySpeed = 1 << 13,
//     WalkSpeed = 1 << 14,
//     Muted = 1 << 15,
//     WorldBuilder = 1 << 16,
//     NoClip = 1 << 17,
//     Count = 1 << 18,
// }

/// Allows a client to place blocks.
pub const ABILITY_BUILD: u32 = 1 << 0;
/// Allows a client to destroy blocks.
pub const ABILITY_MINE: u32 = 1 << 1;
/// Allows a client to interact with doors and switches.
pub const ABILITY_DOORS_AND_SWITCHES: u32 = 1 << 2;
/// Allows a client to open containers.
pub const ABILITY_OPEN_CONTAINERS: u32 = 1 << 3;
/// Allows a client to attack other players.
pub const ABILITY_ATTACK_PLAYERS: u32 = 1 << 4;
/// Allows a client to attack mobs.
pub const ABILITY_ATTACK_MOBS: u32 = 1 << 5;
/// Allows a client to execute operator commands.
pub const ABILITY_OPERATOR_COMMANDS: u32 = 1 << 6;
/// Allows a client to teleport.
/// 
/// I'm not sure why this ability is separate from the operator commands one.
pub const ABILITY_TELEPORT: u32 = 1 << 7;
/// Makes a client invulnerable?
pub const ABILITY_INVULNERABLE: u32 = 1 << 8;
/// Set when the client is currently flying.
pub const ABILITY_FLYING: u32 = 1 << 9;
/// Allows the client to fly.
pub const ABILITY_MAYFLY: u32 = 1 << 10;
/// Not sure what this does.
pub const ABILITY_INSTANT_BUILD: u32 = 1 << 11;
/// Not sure what this does.
pub const ABILITY_LIGHTNING: u32 = 1 << 12;
/// Used to set the fly speed of the client.
pub const ABILITY_FLY_SPEED: u32 = 1 << 13;
/// Used to set the walk speed of the client.
pub const ABILITY_WALK_SPEED: u32 = 1 << 14;
/// Mutes the player. This disables chat client-side and the client will refuse to send chat messages to the server.
pub const ABILITY_MUTED: u32 = 1 << 15;
/// Grants the world builder ability.
pub const ABILITY_WORLD_BUILDER: u32 = 1 << 16;
/// Not sure what this does. Possibly related to spectator mode?
pub const ABILITY_NOCLIP: u32 = 1 << 17;
/// Not sure what this does.
pub const ABILITY_PRIVILEGED_BUILDER: u32 = 1 << 18;
/// Indicates the highest value of the abilities.
pub const ABILITY_FLAG_END: u32 = 1 << 19;

/// Type of ability.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u16)]
#[variant_count]
pub enum AbilityType {
    /// No idea what this is.
    CustomCache,
    /// Used for most abilities.
    Base,
    /// Abilities for spectator.
    Spectator,
    /// Command-related abilities.
    Commands,
    /// Abilities in editor mode.
    Editor,
}

impl TryFrom<u16> for AbilityType {
    type Error = anyhow::Error;

    fn try_from(value: u16) -> anyhow::Result<AbilityType> {
        if value <= AbilityType::variant_count() as u16 {
            // SAFETY: This is safe because the enum has the correct representation and
            // the discriminant is in range.
            Ok(unsafe {
                std::mem::transmute::<u16, AbilityType>(value)
            })
        } else {
            anyhow::bail!("Ability type out of range <= 4, got {value}")
        }
    }
}

/// A single layer in the ability data.
#[derive(Debug, Clone)]
pub struct AbilityLayer {
    /// Type of ability layer.
    pub ability_type: AbilityType,
    /// Enabled abilities for this layer.
    pub abilities: u32,
    /// Values of the abilities.
    pub values: u32,
    /// Default fly speed.
    pub fly_speed: f32,
    /// Default walk speed.
    pub walk_speed: f32,
}

impl Serialize for AbilityLayer {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_u16_le(self.ability_type as u16)?;
        writer.write_u32_le(self.abilities)?;
        writer.write_u32_le(self.values)?;
        writer.write_f32_le(self.fly_speed)?;
        writer.write_f32_le(self.walk_speed)
    }
}

impl<'a> Deserialize<'a> for AbilityLayer {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<AbilityLayer> {
        let ability_type = AbilityType::try_from(reader.read_u16_le()?)?;
        let abilities = reader.read_u32_le()?;
        let values = reader.read_u32_le()?;
        let fly_speed = reader.read_f32_le()?;
        let walk_speed = reader.read_f32_le()?;

        Ok(AbilityLayer {
            ability_type, abilities, values, fly_speed, walk_speed
        })
    }
}

/// Ability data of the client.
#[derive(Debug, Clone)]
pub struct AbilityData {
    /// Entity unique ID.
    pub unique_id: u64,
    /// Player permission level (visitor, member, operator, etc.)
    /// This affects the icon shown in the player list.
    pub permission_level: PermissionLevel,
    /// The command permission level is separate from the standard level.
    /// This level affects which commands the player is allowed to execute.
    pub command_permission_level: CommandPermissionLevel,
    /// Ability layers.
    pub layers: Vec<AbilityLayer>,
}

impl Serialize for AbilityData {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_u64_le(self.unique_id)?; // For some reason this isn't a varint64.
        writer.write_u8(self.permission_level as u8)?;
        writer.write_u8(self.command_permission_level as u8)?;

        writer.write_u8(self.layers.len() as u8)?;
        for layer in &self.layers {
            layer.serialize_into(writer)?;
        }

        Ok(())
    }
}

impl<'a> Deserialize<'a> for AbilityData {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let unique_id = reader.read_u64_le()?;
        let permission_level = PermissionLevel::try_from(reader.read_u8()?)?;
        let command_permission_level = CommandPermissionLevel::try_from(reader.read_u8()?)?;

        let layer_count = reader.read_u8()?;
        // let mut layers = Vec::with_capacity(layers_len as usize);

        let layers = (0..layer_count)
            .map(|_| AbilityLayer::deserialize_from(reader))
            .collect::<anyhow::Result<Vec<_>>>()?;

        Ok(Self {
            unique_id, permission_level, command_permission_level, layers
        })
    }
}

/// Updates the abilities of a user. 
/// 
/// These are the abilities listed in [`AbilityData`].
#[derive(Debug)]
pub struct UpdateAbilities(pub AbilityData);

impl ConnectedPacket for UpdateAbilities {
    const ID: u32 = 0xbb;
}

impl Serialize for UpdateAbilities {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        self.0.serialize_into(writer)
    }
}