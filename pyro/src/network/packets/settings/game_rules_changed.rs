use std::{any::TypeId, fmt};
use std::fmt::Write;


use util::{Serialize, Result, bail};
use util::bytes::{BinaryWriter, MutableBuffer, size_of_varint, VarInt, VarString};

use crate::{command::ParsedArgument, network::packets::ConnectedPacket};

pub const BOOLEAN_GAME_RULES: &[&str] = &[
    "commandblocksenabled",
    "commandblockoutput",
    "dodaylightcycle",
    "doentitydrops",
    "dofiretick",
    "doimmediaterespawn",
    "doinsomnia",
    "domobloot",
    "domobspawning",
    "dotiledrops",
    "doweathercycle",
    "drowningdamage",
    "falldamage",
    "firedamage",
    "freezedamage",
    "keepinventory",
    "mobgriefing",
    "naturalregeneration",
    "pvp",
    "respawnblocksexplode",
    "sendcommandfeedback",
    "showbordereffect",
    "showcoordinates",
    "showdeathmessages",
    "showtags",
    "tntexplodes"
];

pub const INTEGER_GAME_RULES: &[&str] = &[
    "functioncommandlimit",
    "maxcommandchainlength",
    "randomtickspeed",
    "spawnradius"
];

/// Minecraft game rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameRule {
    CommandBlocksEnabled(bool),
    CommandBlockOutput(bool),
    DaylightCycle(bool),
    EntityDrops(bool),
    FireTick(bool),
    Insomnia(bool),
    ImmediateRespawn(bool),
    MobLoot(bool),
    MobSpawning(bool),
    TileDrops(bool),
    WeatherCycle(bool),
    DrowningDamage(bool),
    FallDamage(bool),
    FireDamage(bool),
    FreezeDamage(bool),
    FunctionCommandLimit(i32),
    KeepInventory(bool),
    MaxCommandChainLength(i32),
    MobGriefing(bool),
    NaturalRegeneration(bool),
    Pvp(bool),
    RandomTickSpeed(i32),
    RespawnBlocksExplode(bool),
    SendCommandFeedback(bool),
    ShowBorderEffect(bool),
    ShowCoordinates(bool),
    ShowDeathMessages(bool),
    ShowTags(bool),
    SpawnRadius(i32),
    TntExplodes(bool),
}

impl fmt::Display for GameRule {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::CommandBlocksEnabled(b)
            | Self::CommandBlockOutput(b)
            | Self::DaylightCycle(b)
            | Self::EntityDrops(b)
            | Self::FireTick(b)
            | Self::Insomnia(b)
            | Self::ImmediateRespawn(b)
            | Self::MobLoot(b)
            | Self::MobSpawning(b)
            | Self::TileDrops(b)
            | Self::WeatherCycle(b)
            | Self::DrowningDamage(b)
            | Self::FallDamage(b)
            | Self::FireDamage(b)
            | Self::FreezeDamage(b)
            | Self::KeepInventory(b)
            | Self::MobGriefing(b)
            | Self::NaturalRegeneration(b)
            | Self::Pvp(b)
            | Self::RespawnBlocksExplode(b)
            | Self::SendCommandFeedback(b)
            | Self::ShowBorderEffect(b)
            | Self::ShowCoordinates(b)
            | Self::ShowDeathMessages(b)
            | Self::ShowTags(b)
            | Self::TntExplodes(b) => {
                write!(fmt, "{b}")
            }
            Self::FunctionCommandLimit(i)
            | Self::MaxCommandChainLength(i)
            | Self::RandomTickSpeed(i)
            | Self::SpawnRadius(i) => {
                write!(fmt, "{i}")
            }
        }
    }
}

impl GameRule {
    pub fn is_bool(&self) -> bool {
        match self {
            Self::CommandBlocksEnabled(_)
            | Self::CommandBlockOutput(_)
            | Self::DaylightCycle(_)
            | Self::EntityDrops(_)
            | Self::FireTick(_)
            | Self::Insomnia(_)
            | Self::ImmediateRespawn(_)
            | Self::MobLoot(_)
            | Self::MobSpawning(_)
            | Self::TileDrops(_)
            | Self::WeatherCycle(_)
            | Self::DrowningDamage(_)
            | Self::FallDamage(_)
            | Self::FireDamage(_)
            | Self::FreezeDamage(_)
            | Self::KeepInventory(_)
            | Self::MobGriefing(_)
            | Self::NaturalRegeneration(_)
            | Self::Pvp(_)
            | Self::RespawnBlocksExplode(_)
            | Self::SendCommandFeedback(_)
            | Self::ShowBorderEffect(_)
            | Self::ShowCoordinates(_)
            | Self::ShowDeathMessages(_)
            | Self::ShowTags(_)
            | Self::TntExplodes(_) => {
                true
            }
            Self::FunctionCommandLimit(_)
            | Self::MaxCommandChainLength(_)
            | Self::RandomTickSpeed(_)
            | Self::SpawnRadius(_) => {
                false
            }
        }
    }

    pub fn serialized_size(&self) -> usize {
        self.name().var_len() + match self {
            Self::CommandBlocksEnabled(_)
            | Self::CommandBlockOutput(_)
            | Self::DaylightCycle(_)
            | Self::EntityDrops(_)
            | Self::FireTick(_)
            | Self::Insomnia(_)
            | Self::ImmediateRespawn(_)
            | Self::MobLoot(_)
            | Self::MobSpawning(_)
            | Self::TileDrops(_)
            | Self::WeatherCycle(_)
            | Self::DrowningDamage(_)
            | Self::FallDamage(_)
            | Self::FireDamage(_)
            | Self::FreezeDamage(_)
            | Self::KeepInventory(_)
            | Self::MobGriefing(_)
            | Self::NaturalRegeneration(_)
            | Self::Pvp(_)
            | Self::RespawnBlocksExplode(_)
            | Self::SendCommandFeedback(_)
            | Self::ShowBorderEffect(_)
            | Self::ShowCoordinates(_)
            | Self::ShowDeathMessages(_)
            | Self::ShowTags(_)
            | Self::TntExplodes(_) => {
                1 + 1
            }
            Self::FunctionCommandLimit(v)
            | Self::MaxCommandChainLength(v)
            | Self::RandomTickSpeed(v)
            | Self::SpawnRadius(v) => {
                1 + v.var_len()
            }
        }
    }

    pub fn from_parsed(name: &str, value: &ParsedArgument) -> Result<GameRule> {
        if let ParsedArgument::String(str_boolean) = value {
            let rule_value = match str_boolean.as_str() {
                "true" => true,
                "false" => false,
                _ => bail!(InvalidCommand, "Invalid boolean, must be true or false, got {str_boolean}")
            };

            Ok(match name {
                "commandblocksenabled" => Self::CommandBlocksEnabled(rule_value),
                "commandblockoutput" => Self::CommandBlockOutput(rule_value),
                "dodaylightcycle" => Self::DaylightCycle(rule_value),
                "doentitydrops" => Self::EntityDrops(rule_value),
                "dofiretick" => Self::FireTick(rule_value),
                "doimmediaterespawn" => Self::ImmediateRespawn(rule_value),
                "doinsomnia" => Self::Insomnia(rule_value),
                "domobloot" => Self::MobLoot(rule_value),
                "domobspawning" => Self::MobSpawning(rule_value),
                "dotiledrops" => Self::TileDrops(rule_value),
                "doweathercycle" => Self::WeatherCycle(rule_value),
                "drowningdamage" => Self::DrowningDamage(rule_value),
                "falldamage" => Self::FallDamage(rule_value),
                "firedamage" => Self::FireDamage(rule_value),
                "freezedamage" => Self::FreezeDamage(rule_value),
                "keepinventory" => Self::KeepInventory(rule_value),
                "mobgriefing" => Self::MobGriefing(rule_value),
                "naturalregeneration" => Self::NaturalRegeneration(rule_value),
                "pvp" => Self::Pvp(rule_value),
                "respawnblocksexplode" => Self::RespawnBlocksExplode(rule_value),
                "sendcommandfeedback" => Self::SendCommandFeedback(rule_value),
                "showbordereffect" => Self::ShowBorderEffect(rule_value),
                "showcoordinates" => Self::ShowCoordinates(rule_value),
                "showdeathmessages" => Self::ShowDeathMessages(rule_value),
                "showtags" => Self::ShowTags(rule_value),
                "tntexplodes" => Self::TntExplodes(rule_value),
                _ => bail!(InvalidCommand, "Invalid boolean game rule name {name}")
            })
        } else if let ParsedArgument::Int(integer) = value {
            Ok(match name {
                "functioncommandlimit" => Self::FunctionCommandLimit(*integer),
                "maxcommandchainlength" => Self::MaxCommandChainLength(*integer),
                "randomtickspeed" => Self::RandomTickSpeed(*integer),
                "spawnradius" => Self::SpawnRadius(*integer),
                _ => bail!(InvalidCommand, "Invalid integer game rule name {name}")
            })
        } else {
            bail!(InvalidCommand, "Invalid game rule value type, it must be a boolean or integer")
        }
    }

    pub fn serialize(&self, buffer: &mut MutableBuffer) {
        buffer.write_str(self.name());
        buffer.write_bool(true); // Player can modify. Doesn't seem to do anything.

        match self {
            Self::CommandBlocksEnabled(b)
            | Self::CommandBlockOutput(b)
            | Self::DaylightCycle(b)
            | Self::EntityDrops(b)
            | Self::FireTick(b)
            | Self::Insomnia(b)
            | Self::ImmediateRespawn(b)
            | Self::MobLoot(b)
            | Self::MobSpawning(b)
            | Self::TileDrops(b)
            | Self::WeatherCycle(b)
            | Self::DrowningDamage(b)
            | Self::FallDamage(b)
            | Self::FireDamage(b)
            | Self::FreezeDamage(b)
            | Self::KeepInventory(b)
            | Self::MobGriefing(b)
            | Self::NaturalRegeneration(b)
            | Self::Pvp(b)
            | Self::RespawnBlocksExplode(b)
            | Self::SendCommandFeedback(b)
            | Self::ShowBorderEffect(b)
            | Self::ShowCoordinates(b)
            | Self::ShowDeathMessages(b)
            | Self::ShowTags(b)
            | Self::TntExplodes(b) => {
                buffer.write_var_u32(1);
                buffer.write_bool(*b);
            }
            Self::FunctionCommandLimit(i)
            | Self::MaxCommandChainLength(i)
            | Self::RandomTickSpeed(i)
            | Self::SpawnRadius(i) => {
                buffer.write_var_u32(2);
                buffer.write_var_u32(*i as u32);
            }
        }
    }

    /// Returns the in-game name of the game rule.
    #[inline]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::CommandBlocksEnabled(_) => "commandblocksenabled",
            Self::CommandBlockOutput(_) => "commandblockoutput",
            Self::DaylightCycle(_) => "dodaylightcycle",
            Self::EntityDrops(_) => "doentitydrops",
            Self::FireTick(_) => "dofiretick",
            Self::ImmediateRespawn(_) => "doimmediaterespawn",
            Self::Insomnia(_) => "doinsomnia",
            Self::MobLoot(_) => "domobloot",
            Self::MobSpawning(_) => "domobspawning",
            Self::TileDrops(_) => "dotiledrops",
            Self::WeatherCycle(_) => "doweathercycle",
            Self::DrowningDamage(_) => "drowningdamage",
            Self::FallDamage(_) => "falldamage",
            Self::FireDamage(_) => "firedamage",
            Self::FreezeDamage(_) => "freezedamage",
            Self::FunctionCommandLimit(_) => "functioncommandlimit",
            Self::KeepInventory(_) => "keepinventory",
            Self::MaxCommandChainLength(_) => "maxcommandchainlength",
            Self::MobGriefing(_) => "mobgriefing",
            Self::NaturalRegeneration(_) => "naturalregeneration",
            Self::Pvp(_) => "pvp",
            Self::RandomTickSpeed(_) => "randomtickspeed",
            Self::RespawnBlocksExplode(_) => "respawnblocksexplode",
            Self::SendCommandFeedback(_) => "sendcommandfeedback",
            Self::ShowBorderEffect(_) => "showbordereffect",
            Self::ShowCoordinates(_) => "showcoordinates",
            Self::ShowDeathMessages(_) => "showdeathmessages",
            Self::ShowTags(_) => "showtags",
            Self::SpawnRadius(_) => "spawnradius",
            Self::TntExplodes(_) => "tntexplodes",
        }
    }
}

/// Updates one or more game rules.
#[derive(Debug, Clone)]
pub struct GameRulesChanged<'a> {
    /// Game rules to update.
    pub game_rules: &'a [GameRule],
}

impl ConnectedPacket for GameRulesChanged<'_> {
    const ID: u32 = 0x48;

    fn serialized_size(&self) -> usize {
        size_of_varint(self.game_rules.len() as u32) + 
        self.game_rules.iter().fold(
            0, |acc, g| acc + 1 + if g.is_bool() { 1 } else { 4 }
        )
    }
}

impl Serialize for GameRulesChanged<'_> {
    fn serialize(&self, buffer: &mut MutableBuffer) {
        buffer.write_var_u32(self.game_rules.len() as u32);
        for game_rule in self.game_rules {
            game_rule.serialize(buffer);
        }
    }
}
