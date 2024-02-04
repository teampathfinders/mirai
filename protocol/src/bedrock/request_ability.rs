use util::{bail};
use util::{BinaryRead};
use util::Deserialize;

use crate::bedrock::ConnectedPacket;

/// Ability type together with its value.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Ability {
    /// Allows the client to place blocks.
    Build(bool),
    /// Allows the client to break blocks.
    Mine(bool),
    /// Allows the client to use doors and switches.
    DoorsAndSwitches(bool),
    /// Allows the client to open containers.
    OpenContainers(bool),
    /// Allows the client to attack other players.
    AttackPlayers(bool),
    /// Allows the client to attack mobs.
    AttackMobs(bool),
    /// Allows to client to execute operator commands.
    OperatorCommands(bool),
    /// Allows the client to teleport.
    /// 
    /// Not sure why this is separate from operator commands.
    Teleport(bool),
    /// Makes the client invulnerable? Maybe this is related to creative mode.
    Invulnerable(bool),
    /// Set when the client is currently flying.
    Flying(f32),
    /// Allows the client to fly.
    MayFly(bool),
    /// Not sure what this is...
    InstantBuild(bool),
    /// Not sure what this is...
    Lightning(bool),
    /// Sets the fly speed of the client.
    FlySpeed(f32),
    /// Sets the walk speed of the client.
    WalkSpeed(f32),
    /// Set to true when the client is muted.
    /// 
    /// This will prevent the client from sending any chat messages to the server.
    /// If the user tries to send a message, the game will say chat is disabled.
    Muted(bool),
    /// Allows a player to build in the world.
    WorldBuilder(bool),
    /// Enables noclip
    NoClip(bool),
    /// Not sure what this does.
    Count(f32),
}

/// Sent by the client to request permission to use a specific ability.
#[derive(Debug)]
pub struct RequestAbility {
    /// Ability to request.
    pub ability: Ability,
}

impl ConnectedPacket for RequestAbility {
    const ID: u32 = 0xb8;
}

impl<'a> Deserialize<'a> for RequestAbility {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let ability_type = reader.read_var_i32()?;
        let value_type = reader.read_u8()?;

        let mut bool_value = false;
        let mut float_value = 0.0;

        if value_type == 1 {
            bool_value = reader.read_bool()?;
        } else if value_type == 2 {
            float_value = reader.read_f32_be()?;
        } else {
            bail!(Malformed, "Invalid ability value type {value_type}")
        }

        Ok(Self {
            ability: match ability_type {
                0 => Ability::Build(bool_value),
                1 => Ability::Mine(bool_value),
                2 => Ability::DoorsAndSwitches(bool_value),
                3 => Ability::OpenContainers(bool_value),
                4 => Ability::AttackPlayers(bool_value),
                5 => Ability::AttackMobs(bool_value),
                6 => Ability::OperatorCommands(bool_value),
                7 => Ability::Teleport(bool_value),
                8 => Ability::Invulnerable(bool_value),
                9 => Ability::Flying(float_value),
                10 => Ability::MayFly(bool_value),
                11 => Ability::InstantBuild(bool_value),
                12 => Ability::Lightning(bool_value),
                13 => Ability::FlySpeed(float_value),
                14 => Ability::WalkSpeed(float_value),
                15 => Ability::Muted(bool_value),
                16 => Ability::WorldBuilder(bool_value),
                17 => Ability::NoClip(bool_value),
                18 => Ability::Count(float_value),
                _ => bail!(Malformed, "Invalid ability type {ability_type}"),
            },
        })
    }
}
