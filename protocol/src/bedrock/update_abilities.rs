use util::{BinaryRead, BinaryWrite, MutableBuffer, SharedBuffer};
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

pub const ABILITY_BUILD: u32 = 1 << 0;
pub const ABILITY_MINE: u32 = 1 << 1;
pub const ABILITY_DOORS_AND_SWITCHES: u32 = 1 << 2;
pub const ABILITY_OPEN_CONTAINERS: u32 = 1 << 3;
pub const ABILITY_ATTACK_PLAYERS: u32 = 1 << 4;
pub const ABILITY_ATTACK_MOBS: u32 = 1 << 5;
pub const ABILITY_OPERATOR_COMMANDS: u32 = 1 << 6;
pub const ABILITY_TELEPORT: u32 = 1 << 7;
pub const ABILITY_INVULNERABLE: u32 = 1 << 8;
pub const ABILITY_FLYING: u32 = 1 << 9;
pub const ABILITY_MAYFLY: u32 = 1 << 10;
pub const ABILITY_INSTANT_BUILD: u32 = 1 << 11;
pub const ABILITY_LIGHTNING: u32 = 1 << 12;
pub const ABILITY_FLY_SPEED: u32 = 1 << 13;
pub const ABILITY_WALK_SPEED: u32 = 1 << 14;
pub const ABILITY_MUTED: u32 = 1 << 15;
pub const ABILITY_WORLD_BUILDER: u32 = 1 << 16;
pub const ABILITY_NOCLIP: u32 = 1 << 17;
pub const ABILITY_PRIVILEGED_BUILDER: u32 = 1 << 18;
pub const ABILITY_FLAG_END: u32 = 1 << 19;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u16)]
pub enum AbilityType {
    CustomCache,
    Base,
    Spectator,
    Commands,
    Editor,
}

impl TryFrom<u16> for AbilityType {
    type Error = anyhow::Error;

    fn try_from(value: u16) -> anyhow::Result<Self> {
        if value <= 4 {
            Ok(unsafe {
                std::mem::transmute::<u16, AbilityType>(value)
            })
        } else {
            anyhow::bail!("Ability type out of range <= 4, got {value}")
        }
    }
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
    fn serialize(&self, buffer: &mut MutableBuffer) -> anyhow::Result<()> {
        buffer.write_u16_le(self.ability_type as u16)?;
        buffer.write_u32_le(self.abilities)?;
        buffer.write_u32_le(self.values)?;
        buffer.write_f32_le(self.fly_speed)?;
        buffer.write_f32_le(self.walk_speed)
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
    pub layers: Vec<AbilityLayer>,
}

impl Serialize for AbilityData {
    fn serialize(&self, buffer: &mut MutableBuffer) -> anyhow::Result<()> {
        buffer.write_u64_le(self.unique_id)?; // For some reason this isn't a varint64.
        buffer.write_u8(self.permission_level as u8)?;
        buffer.write_u8(self.command_permission_level as u8)?;

        buffer.write_u8(self.layers.len() as u8)?;
        for layer in &self.layers {
            layer.serialize(buffer)?;
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

#[derive(Debug)]
pub struct UpdateAbilities(pub AbilityData);

impl ConnectedPacket for UpdateAbilities {
    const ID: u32 = 0xbb;
}

impl Serialize for UpdateAbilities {
    #[inline]
    fn serialize(&self, buffer: &mut MutableBuffer) -> anyhow::Result<()> {
        self.0.serialize(buffer)
    }
}