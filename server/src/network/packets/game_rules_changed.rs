use bytes::BytesMut;
use common::{Encodable, VResult, WriteExtensions};

use super::GamePacket;

/// Minecraft game rules.
#[derive(Debug)]
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
    FunctionCommandLimit(u32),
    KeepInventory(bool),
    MaxCommandChainLength(u32),
    MobGriefing(bool),
    NaturalRegeneration(bool),
    Pvp(bool),
    RandomTickSpeed(u32),
    RespawnBlocksExplode(bool),
    SendCommandFeedback(bool),
    ShowBorderEffect(bool),
    ShowCoordinates(bool),
    ShowDeathMessages(bool),
    ShowTags(bool),
    SpawnRadius(u32),
    TntExplodes(bool),
}

impl GameRule {
    pub fn encode(&self, buffer: &mut BytesMut) {
        buffer.put_string(self.name());
        buffer.put_bool(true); // Player can modify. Doesn't seem to do anything.


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
                buffer.put_var_u32(1);
                buffer.put_bool(*b);
            }
            Self::FunctionCommandLimit(i)
            | Self::MaxCommandChainLength(i)
            | Self::RandomTickSpeed(i)
            | Self::SpawnRadius(i) => {
                buffer.put_var_u32(2);
                buffer.put_var_u32(*i);
            }
        }
    }

    /// Returns the in-game name of the game rule.
    #[inline]
    pub fn name(&self) -> &'static str {
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
            Self::TntExplodes(_) => "tntexplodes"
        }
    }
}

/// Updates one or more game rules.
#[derive(Debug)]
pub struct GameRulesChanged {
    /// Game rules to update.
    pub game_rules: Vec<GameRule>,
}

impl GamePacket for GameRulesChanged {
    const ID: u32 = 0x48;
}

impl Encodable for GameRulesChanged {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_var_u32(self.game_rules.len() as u32);
        for game_rule in &self.game_rules {
            game_rule.encode(&mut buffer);
        }

        Ok(buffer)
    }
}
