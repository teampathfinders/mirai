use util::{BinaryWrite, size_of_varint};

use util::Serialize;

use crate::bedrock::ConnectedPacket;

/// Operation to perform with the effect.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MobEffectAction {
    /// Do nothing.
    None,
    /// Adds an effect to an entity.
    Add,
    /// Modifies an entity's effect.
    Modify,
    /// Removes an effect from an entity.
    Remove,
}

/// Type of effect to apply.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MobEffectKind {
    /// Increases the speed of the player.
    Speed = 1,
    /// Slows the player down.
    Slowness,
    /// Increases mining speed.
    Haste,
    /// Decreases mining speed.
    MiningFatigue,
    /// Increases damage per hit.
    Strength,
    /// Instantly adds a health to the player.
    InstantHealth,
    /// Instantly damages the player.
    InstantDamage,
    /// Increases the player's jump height.
    JumpBoost,
    /// Causes the nausea effect.
    Nausea,
    /// Slowly regenerates the player's health.
    Regeneration,
    /// Makes the player more resistant against damage.
    Resistance,
    /// Makes the player completely resistance against fire damage.
    FireResistance,
    /// Allows the player to breathe underwater.
    WaterBreathing,
    /// Makes the player invisible.
    Invisibility,
    /// Makes the player blind.
    Blindness,
    /// Improves vision in the dark.
    NightVision,
    /// 
    Hunger,
    /// Decrease damage per hit.
    Weakness,
    /// Slowly damages the player but is unable to kill them.
    Poison,
    /// Similar to poison but acts slower and can kill.
    Wither,
    /// Increases the maximum health of the player. Unlike absorption, these hearts can be replenished by food.
    HealthBoost,
    /// Adds yellow hearts to your healthbar that cannot be replenished by food.
    Absorption,
    /// Makes the player able to regenerate naturally.
    Saturation,
    /// Makes the player levitate into the air.
    /// 
    /// This effect is given by Shulkers.
    Levitation,
    /// Similar to poison but can kill.
    FatalPoison,
    /// Effect given by a conduit when it is active.
    ConduitPower,
    /// Makes the player slowly fall down.
    SlowFalling,
    /// Causes a raid when the player enters a village.
    BadOmen,
    /// Allows the player to receive gifts from villagers after defeating a raid.
    HeroOfTheVillage,
    /// Effect given to the player when in vicinity of a Warden.
    Darkness,
}

/// String names of the effects.
pub const MOBEFFECT_NAMES: &[&str] = &[
    "absorption",
    "bad_omen",
    "blindness",
    "conduit_power",
    "darkness",
    "fatal_poison",
    "fire_resistance",
    "haste",
    "health_boost",
    "hunger",
    "instant_damage",
    "invisibility",
    "jump_boost",
    "levitation",
    "mining_fatigue",
    "nausea",
    "night_vision",
    "poison",
    "regeneration",
    "resistance",
    "saturation",
    "slow_falling",
    "slowness",
    "speed",
    "strength",
    "village_hero",
    "water_breathing",
    "weakness",
    "wither"
];

/// Updates entity effects.
#[derive(Debug, Clone)]
pub struct MobEffectUpdate {
    /// Runtime ID of the affected entity.
    pub runtime_id: u64,
    /// Operation to perform on the entity.
    pub action: MobEffectAction,
    /// Type of effect.
    pub effect_kind: MobEffectKind,
    /// Strength of the effect, this ranges from 0-255.
    pub amplifier: i32,
    /// Whether to display particles.
    pub particles: bool,
    /// Duration of the effect in ticks.
    pub duration: i32,
}

impl ConnectedPacket for MobEffectUpdate {
    const ID: u32 = 0x1c;

    fn serialized_size(&self) -> usize {
        size_of_varint(self.runtime_id) + 1 +
            size_of_varint(self.effect_kind as i32) +
            size_of_varint(self.amplifier) + 1 +
            size_of_varint(self.duration)
    }
}

impl Serialize for MobEffectUpdate {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_var_u64(self.runtime_id)?;
        writer.write_u8(self.action as u8)?;
        writer.write_var_i32(self.effect_kind as i32)?;
        writer.write_var_i32(self.amplifier)?;
        writer.write_bool(self.particles)?;
        writer.write_var_i32(self.duration)
    }
}
