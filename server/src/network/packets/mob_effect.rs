use bytes::{BufMut, BytesMut};
use common::{VResult, WriteExtensions};

use crate::network::Encodable;

use super::GamePacket;

#[derive(Debug, Copy, Clone)]
pub enum MobEffectOperation {
    None,
    Add,
    Modify,
    Remove,
}

impl MobEffectOperation {
    pub fn encode(&self, buffer: &mut BytesMut) {
        buffer.put_u8(*self as u8);
    }
}

#[derive(Debug, Copy, Clone)]
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

impl MobEffectKind {
    pub fn encode(&self, buffer: &mut BytesMut) {
        buffer.put_var_i32(*self as i32);
    }
}

#[derive(Debug)]
pub struct MobEffectUpdate {
    pub runtime_id: u64,
    pub operation: MobEffectOperation,
    pub effect_kind: MobEffectKind,
    pub amplifier: i32,
    pub particles: bool,
    pub duration: i32,
}

impl GamePacket for MobEffectUpdate {
    const ID: u32 = 0x1c;
}

impl Encodable for MobEffectUpdate {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_var_u64(self.runtime_id);
        self.operation.encode(&mut buffer);
        self.effect_kind.encode(&mut buffer);
        buffer.put_var_i32(self.amplifier);
        buffer.put_bool(self.particles);
        buffer.put_var_i32(self.duration);

        Ok(buffer)
    }
}
