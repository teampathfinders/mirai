use bytes::{BytesMut, Buf};
use common::{VResult, VError, ReadExtensions, bail};

use crate::network::Decodable;

use super::GamePacket;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Ability {
    Build(bool),
    Mine(bool),
    DoorsAndSwitches(bool),
    OpenContainers(bool),
    AttackPlayers(bool),
    AttackMobs(bool),
    OperatorCommands(bool),
    Teleport(bool),
    Invulnerable(bool),
    Flying(f32),
    MayFly(bool),
    InstantBuild(bool),
    Lightning(bool),
    FlySpeed(f32),
    WalkSpeed(f32),
    Muted(bool),
    WorldBuilder(bool),
    NoClip(bool),
    Count(f32)
}

/// Sent by the client to request permission to use a specific ability.
#[derive(Debug)]
pub struct RequestAbility {
    pub ability: Ability
}

impl GamePacket for RequestAbility {
    const ID: u32 = 0xb8;
}

impl Decodable for RequestAbility {
    fn decode(mut buffer: BytesMut) -> VResult<Self> {
        let ability_type = buffer.get_var_i32()?;
        let value_type = buffer.get_u8();
        
        let mut bool_value = false;
        let mut float_value = 0.0;

        tracing::info!("{value_type}");

        if value_type == 1 {
            bool_value = buffer.get_bool();
        } else if value_type == 2 {
            float_value = buffer.get_f32();
        } else {
            bail!(BadPacket, "Invalid ability value type {value_type}")
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
                12 => Ability::Lightning(todo!()),
                13 => Ability::FlySpeed(float_value),
                14 => Ability::WalkSpeed(float_value),
                15 => Ability::Muted(bool_value),
                16 => Ability::WorldBuilder(bool_value),
                17 => Ability::NoClip(bool_value),
                18 => Ability::Count(float_value),
                _ => bail!(BadPacket, "Invalid ability type {ability_type}")
            }
        })
    }
}