

use level::Dimension;
use crate::ConnectedPacket;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u16)]
pub enum CopperWaxType {
    WaxUnoxidised = 0xa609,
    WaxExposed = 0xa809,
    WaxWeathered = 0xaa09,
    WaxOxidised = 0xac09,
    UnwaxUnoxidised = 0xae09,
    UnwaxExposed = 0xb009,
    UnwaxWeathered = 0xb209,
    UnwaxOxidised = 0xfa0a
}

#[derive(Debug, Clone)]
pub enum EventType {
    AchievementAwarded {
        achievement_id: i32
    },
    // TODO
    EntityInteract {
        interaction_type: i32,
        entity_type: i32,
        variant: i32,
        color: u8
    },
    PortalBuilt {
        dimension: Dimension
    },
    PortalUsed {
        from: Dimension,
        to: Dimension
    },
    MobKilled {
        killer_unique_id: i64,
        victim_unique_id: i64,
        killer_type: i32,
        damage_cause: i32,
        villager_trade_tier: i32,
        villager_display_name: String
    },
    CauldronUsed {
        potion_id: i32,
        color: i32,
        fill_level: i32
    },
    PlayerDied {
        attacker_unique_id: i32,
        attacker_variant: i32,
        damage_cause: i32,
        in_raid: bool
    },
    BossKilled {
        boss_unique_id: i32,
        party_size: i32,
        entity_type: i32
    },
    AgentCommand {
        result: i32,
        value: i32,
        command: String,
        data_key: String,
        output: String
    },
    PatternRemoved {
        item_id: i32,
        aux_value: i32,
        pattern_size: i32,
        pattern_index: i32,
        pattern_color: i32
    },
    SlashCommandExecuted {
        command_name: String,
        success_count: i32,
        message_count: i32,
        output: String
    },
    FishBucketed {
        pattern: i32,
        preset: i32,
        entity_type: i32,
        release: bool
    },
    MobBorn {
        entity_type: i32,
        variant: i32,
        color: u8
    },
    PetDied {
        killed_by_owner: bool,
        killer_unique_id: i64,
        pet_unique_id: i64,
        damage_cause: i32,
        entity_type: i32
    },
    CauldronInteract {
        interaction_type: i32,
        item_id: i32
    },
    ComposterInteract {
        interaction_type: i32,
        item_id: i32
    },
    BellUsed {
        item_id: i32
    },
    EntityDefinitionTrigger {
        event: String
    },
    RaidUpdate {
        raid_wave: i32,
        total_raid_waves: i32,
        raid_won: bool
    },
    MovementAnomaly {
        event_type: u8,
        cheating_score: f32,
        average_delta: f32,
        total_delta: f32,
        min_delta: f32,
        max_delta: f32
    },
    MovementCorrected {
        delta: f32,
        cheating_score: f32,
        score_threshold: f32,
        distance_threshold: f32,
        duration_threshold: i32
    },
    ExtractHoney {},
    CopperWaxed {
        wax_type: CopperWaxType
    },
    SneakCloseToSculkSensor {}
}

#[derive(Debug, Clone)]
pub struct Event {
    pub runtime_id: u64,
    pub event: EventType
}

impl ConnectedPacket for Event {
    const ID: u32 = 0x41;
}