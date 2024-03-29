use std::fmt;

use macros::variant_count;
use util::{Serialize, BinaryWrite, size_of_varint, VarInt, VarString};

use crate::bedrock::ConnectedPacket;

// FIXME: This whole module could use some cleanup...

/// Names of all boolean game rules.
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

/// Names of all integer value gamerules.
pub const INTEGER_GAME_RULES: &[&str] = &[
    "functioncommandlimit",
    "maxcommandchainlength",
    "randomtickspeed",
    "spawnradius"
];

/// Minecraft game rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[variant_count]
pub enum GameRule {
    /// Whether command blocks are enabled.
    CommandBlocksEnabled(bool),
    /// Whether command blocks show output in chat.
    CommandBlockOutput(bool),
    /// Whether time changes.
    DaylightCycle(bool),
    /// Whether entities drop items.
    EntityDrops(bool),
    /// Whether fire spreads.
    FireTick(bool),
    /// Whether players can experience insomnia.
    Insomnia(bool),
    /// Whether players immediately respawn without showing the death screen.
    ImmediateRespawn(bool),
    /// Whether mobs drop loot.
    MobLoot(bool),
    /// Whether mobs can spawn.
    MobSpawning(bool),
    /// Whether tile entities drop items.
    TileDrops(bool),
    /// Whether the weather will change.
    WeatherCycle(bool),
    /// Whether players can drown.
    DrowningDamage(bool),
    /// Whether fall damage is enabled.
    FallDamage(bool),
    /// Whether fire damages players.
    FireDamage(bool),
    /// Whether freezing damages players.
    FreezeDamage(bool),
    /// Limit of the total amount of allowed commands in a single function.
    FunctionCommandLimit(i32),
    /// Whether players retain their inventory on death.
    KeepInventory(bool),
    /// Max length of a command block chain.
    MaxCommandChainLength(i32),
    /// Whether mobs can destroy blocks.
    MobGriefing(bool),
    /// Whether players naturally regenerate.
    NaturalRegeneration(bool),
    /// Whether players can attack other players.
    Pvp(bool),
    /// The random tick speed.
    RandomTickSpeed(i32),
    /// Whether respawn blocks can explode.
    RespawnBlocksExplode(bool),
    /// Whether to send command feedback.
    SendCommandFeedback(bool),
    /// Whether to show the border effect.
    ShowBorderEffect(bool),
    /// Whether to show coordinates.
    ShowCoordinates(bool),
    /// Whether to show death messages.
    ShowDeathMessages(bool),
    /// Whether to show tags.
    ShowTags(bool),
    /// Radius around the spawnpoint that players can spawn in.
    SpawnRadius(i32),
    /// Whether TNT can explode.
    TntExplodes(bool),
}

impl GameRule {
    /// Whether the gamerule is a bool type.
    pub const fn is_bool(&self) -> bool {
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

    // /// Creates a [`GameRule`] from a parsed command argument.
    // pub fn from_parsed(name: &str, value: &ParsedArgument) -> anyhow::Result<GameRule> {
    //     if let ParsedArgument::String(str_boolean) = value {
    //         let rule_value = match str_boolean.as_str() {
    //             "true" => true,
    //             "false" => false,
    //             _ => bail!(Malformed, "Invalid boolean, must be true or false, got {str_boolean}")
    //         };

    //         Ok(match name {
    //             "commandblocksenabled" => Self::CommandBlocksEnabled(rule_value),
    //             "commandblockoutput" => Self::CommandBlockOutput(rule_value),
    //             "dodaylightcycle" => Self::DaylightCycle(rule_value),
    //             "doentitydrops" => Self::EntityDrops(rule_value),
    //             "dofiretick" => Self::FireTick(rule_value),
    //             "doimmediaterespawn" => Self::ImmediateRespawn(rule_value),
    //             "doinsomnia" => Self::Insomnia(rule_value),
    //             "domobloot" => Self::MobLoot(rule_value),
    //             "domobspawning" => Self::MobSpawning(rule_value),
    //             "dotiledrops" => Self::TileDrops(rule_value),
    //             "doweathercycle" => Self::WeatherCycle(rule_value),
    //             "drowningdamage" => Self::DrowningDamage(rule_value),
    //             "falldamage" => Self::FallDamage(rule_value),
    //             "firedamage" => Self::FireDamage(rule_value),
    //             "freezedamage" => Self::FreezeDamage(rule_value),
    //             "keepinventory" => Self::KeepInventory(rule_value),
    //             "mobgriefing" => Self::MobGriefing(rule_value),
    //             "naturalregeneration" => Self::NaturalRegeneration(rule_value),
    //             "pvp" => Self::Pvp(rule_value),
    //             "respawnblocksexplode" => Self::RespawnBlocksExplode(rule_value),
    //             "sendcommandfeedback" => Self::SendCommandFeedback(rule_value),
    //             "showbordereffect" => Self::ShowBorderEffect(rule_value),
    //             "showcoordinates" => Self::ShowCoordinates(rule_value),
    //             "showdeathmessages" => Self::ShowDeathMessages(rule_value),
    //             "showtags" => Self::ShowTags(rule_value),
    //             "tntexplodes" => Self::TntExplodes(rule_value),
    //             _ => bail!(Malformed, "Invalid boolean game rule name {name}")
    //         })
    //     } else if let ParsedArgument::Int(integer) = value {
    //         Ok(match name {
    //             "functioncommandlimit" => Self::FunctionCommandLimit(*integer),
    //             "maxcommandchainlength" => Self::MaxCommandChainLength(*integer),
    //             "randomtickspeed" => Self::RandomTickSpeed(*integer),
    //             "spawnradius" => Self::SpawnRadius(*integer),
    //             _ => bail!(Malformed, "Invalid integer game rule name {name}")
    //         })
    //     } else {
    //         bail!(Malformed, "Invalid game rule value type, it must be a boolean or integer")
    //     }
    // }

    pub fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_str(self.name())?;
        writer.write_bool(true)?; // Player can modify. Doesn't seem to do anything.

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
                writer.write_var_u32(1)?;
                writer.write_bool(*b)
            }
            Self::FunctionCommandLimit(i)
            | Self::MaxCommandChainLength(i)
            | Self::RandomTickSpeed(i)
            | Self::SpawnRadius(i) => {
                writer.write_var_u32(2)?;
                writer.write_var_u32(*i as u32)
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
                0, |acc, g| acc + 1 + if g.is_bool() { 1 } else { 4 },
            )
    }
}

impl Serialize for GameRulesChanged<'_> {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_var_u32(self.game_rules.len() as u32)?;
        for game_rule in self.game_rules {
            game_rule.serialize_into(writer)?;
        }

        Ok(())
    }
}
