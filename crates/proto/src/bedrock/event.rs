use crate::types::Dimension;

use crate::bedrock::ConnectedPacket;

/// Type of copper wax.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u16)]
pub enum CopperWaxType {
    /// Waxes unoxidised copper.
    WaxUnoxidized = 0xa609,
    /// Waxes exposed copper.
    WaxExposed = 0xa809,
    /// Waxes weathered copper.
    WaxWeathered = 0xaa09,
    /// Waxes oxidised copper.
    WaxOxidised = 0xac09,
    /// Unwaxes unoxidised copper.
    UnwaxUnoxidized = 0xae09,
    /// Unwaxes exposed copper.
    UnwaxExposed = 0xb009,
    /// Unwaxes weathered copper.
    UnwaxWeathered = 0xb209,
    /// Unwaxes oxidised copper.
    UnwaxOxidized = 0xfa0a,
}

/// Type of event that occurred.
#[derive(Debug, Clone)]
pub enum EventType {
    /// An achievement has been awarded.
    AchievementAwarded {
        /// ID of the awarded achievement.
        achievement_id: i32
    },
    /// Interacted with an entity.
    EntityInteract {
        /// Type of interaction performed.
        interaction_type: i32,
        /// Type of entity.
        entity_type: i32,
        /// Variant of the entity.
        variant: i32,
        color: u8,
    },
    /// A portal has been built.
    PortalBuilt {
        /// The dimension this portal will lead to.
        dimension: Dimension
    },
    /// A portal has been used.
    PortalUsed {
        /// Origin dimension.
        from: Dimension,
        /// Target dimension.
        to: Dimension,
    },
    /// A mob has been killed.
    MobKilled {
        /// Unique ID of the entity that killed this mob.
        killer_unique_id: i64,
        /// Unique ID of the entity that was killed.
        victim_unique_id: i64,
        /// The type of killer.
        killer_type: i32,
        /// The cause of the damage.
        damage_cause: i32,
        /// Trade tier of the village if the killed mob was a villager.
        villager_trade_tier: i32,
        /// Display name of the villager if the killed mob was a villager.
        villager_display_name: String,
    },
    /// A cauldron has been used.
    CauldronUsed {
        /// ID of the potion that was used with the cauldron.
        potion_id: i32,
        /// Colour of the cauldron.
        color: i32,
        /// Fill level of the cauldron.
        fill_level: i32,
    },
    /// A player has died.
    PlayerDied {
        /// The unique ID of the killer of the player.
        attacker_unique_id: i32,
        /// Type of the attacker.
        attacker_variant: i32,
        /// Cause of the damage to the player.
        damage_cause: i32,
        /// Whether the player died during a raid.
        in_raid: bool,
    },
    /// A boss has been killed.
    BossKilled {
        /// Unique ID of the boss.
        boss_unique_id: i32,
        /// How many players were fighting the boss.
        party_size: i32,
        /// Type of boss that was killed.
        entity_type: i32,
    },
    /// Executes an agent command.
    AgentCommand {
        /// Result of the command.
        result: i32,
        /// Not sure what this is.
        value: i32,
        /// Command to execute.
        command: String,
        /// Data key used in the command.
        data_key: String,
        /// Output of the command.
        output: String,
    },
    /// Removes a pattern from a banner.
    PatternRemoved {
        /// Item the pattern was removed from.
        item_id: i32,
        aux_value: i32,
        pattern_size: i32,
        pattern_index: i32,
        pattern_color: i32,
    },
    /// Executes a slash command.
    SlashCommandExecuted {
        /// Name of the command.
        command_name: String,
        /// How many of the outputted messages were successful.
        success_count: i32,
        /// How many messages were outputted.
        message_count: i32,
        /// Output of the command.
        output: String,
    },
    /// A fish was put into a bucket.
    FishBucketed {
        /// Pattern of the fish.
        pattern: i32,
        preset: i32,
        /// Type of entity that was bucketed.
        entity_type: i32,
        /// Whether the fish was released from the bucket.
        release: bool,
    },
    /// A mob was born.
    MobBorn {
        /// Type of mob that was born.
        entity_type: i32,
        /// Variant of the born mob.
        variant: i32,
        /// Colour of the born mob.
        color: u8,
    },
    /// A pet has died.
    PetDied {
        /// Whether the pet was killed by its owner.
        killed_by_owner: bool,
        /// Unique ID of the killer.
        killer_unique_id: i64,
        /// Unique ID of the pet.
        pet_unique_id: i64,
        /// Cause of the damage to the pet.
        damage_cause: i32,
        /// Type of the pet entity.
        entity_type: i32,
    },
    /// Interacted with a cauldron.
    CauldronInteract {
        /// Interaction type performed.
        interaction_type: i32,
        /// Item used in the interaction.
        item_id: i32,
    },
    /// Interacted with a composter.
    ComposterInteract {
        /// Interaction type performed.
        interaction_type: i32,
        /// Item used in the interaction.
        item_id: i32,
    },
    /// Interacted with a bell.
    BellUsed {
        /// Item used on the bell.
        item_id: i32
    },
    EntityDefinitionTrigger {
        event: String
    },
    RaidUpdate {
        raid_wave: i32,
        total_raid_waves: i32,
        raid_won: bool,
    },
    /// Server found an issue with the player's movement.
    MovementAnomaly {
        event_type: u8,
        cheating_score: f32,
        average_delta: f32,
        total_delta: f32,
        min_delta: f32,
        max_delta: f32,
    },
    /// Player movement was corrected by the server.
    MovementCorrected {
        delta: f32,
        cheating_score: f32,
        score_threshold: f32,
        distance_threshold: f32,
        duration_threshold: i32,
    },
    /// Extracted honey from a beehive.
    ExtractHoney {},
    /// Waxed copper.
    CopperWaxed {
        /// Type of operation performed.
        wax_type: CopperWaxType
    },
    /// Sneaked close to a sculk sensor.
    SneakCloseToSculkSensor {},
}

/// A basic event.
#[derive(Debug, Clone)]
pub struct Event {
    /// Runtime ID of the client.
    pub runtime_id: u64,
    /// Event that occurred.
    pub event: EventType,
}

impl ConnectedPacket for Event {
    const ID: u32 = 0x41;
}