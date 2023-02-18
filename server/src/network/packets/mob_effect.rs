use bytes::{BufMut, BytesMut};
use common::{VResult, WriteExtensions, size_of_var};

use common::Serialize;

use super::GamePacket;

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
    Speed = 1,
    Slowness,
    Haste,
    MiningFatigue,
    Strength,
    InstantHealth,
    InstantDamage,
    JumpBoost,
    Nausea,
    Regeneration,
    Resistance,
    FireResistance,
    WaterBreathing,
    Invisibility,
    Blindness,
    NightVision,
    Hunger,
    Weakness,
    Poison,
    Wither,
    HealthBoost,
    Absorption,
    Saturation,
    Levitation,
    FatalPoison,
    ConduitPower,
    SlowFalling,
    BadOmen,
    HeroOfTheVillage,
    Darkness,
}

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

impl GamePacket for MobEffectUpdate {
    const ID: u32 = 0x1c;
}

impl Serialize for MobEffectUpdate {
    fn serialize(&self) -> VResult<BytesMut> {
        let packet_size = 
            size_of_var(self.runtime_id) + 1 +
            size_of_var(self.effect_kind as i32) +
            size_of_var(self.amplifier) + 1 +
            size_of_var(self.duration);

        let mut buffer = BytesMut::with_capacity(packet_size);

        buffer.put_var_u64(self.runtime_id);
        buffer.put_u8(self.action as u8);
        buffer.put_var_i32(self.effect_kind as i32);
        buffer.put_var_i32(self.amplifier);
        buffer.put_bool(self.particles);
        buffer.put_var_i32(self.duration);

        Ok(buffer)
    }
}
