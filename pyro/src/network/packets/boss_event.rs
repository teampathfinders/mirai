
use util::{Serialize, Result};
use util::bytes::{BinaryWrite, MutableBuffer};
use crate::ConnectedPacket;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BossEventColor {
    Grey,
    Blue,
    Red,
    Green,
    Yellow,
    Purple,
    White
}

#[derive(Debug, Clone)]
pub enum BossEventType<'a> {
    Show {
        bar_title: &'a str,
        color: BossEventColor
    },
    RegisterPlayer {
        player_unique_id: i64
    },
    Hide,
    UnregisterPlayer {
        player_unique_id: i64
    },
    HealthPercentage {
        health_percentage: f32
    },
    Title {
        bar_title: &'a str
    },
    AppearanceProperties {
        color: BossEventColor
    },
    Texture {
        color: BossEventColor
    },
    Request {
        player_unique_id: i64
    }
}

#[derive(Debug, Clone)]
pub struct BossEvent<'a> {
    pub boss_unique_id: i64,
    pub event: BossEventType<'a>
}

impl ConnectedPacket for BossEvent<'_> {
    const ID: u32 = 0x4a;
}

impl Serialize for BossEvent<'_> {
    fn serialize(&self, buffer: &mut MutableBuffer) -> Result<()> {
        buffer.write_var_i64(self.boss_unique_id);
        match self.event {
            BossEventType::Show {
                bar_title, color
            } => {
                buffer.write_var_u32(0); // Event type.

                buffer.write_str(bar_title);
                buffer.write_f32_le(0.0); // HealthPercentage is unused.
                buffer.write_i16_le(0); // ScreenDarkening is unused.
                buffer.write_var_u32(color as u32);
                buffer.write_var_u32(0); // Overlay is unused.
            },
            BossEventType::RegisterPlayer {
                player_unique_id
            } => {
                buffer.write_var_u32(1); // Event type.
                buffer.write_var_i64(player_unique_id);
            },
            BossEventType::Hide => {
                buffer.write_var_u32(2); // Event type.
            },
            BossEventType::UnregisterPlayer {
                player_unique_id
            } => {
                buffer.write_var_u32(3); // Event type.
                buffer.write_var_i64(player_unique_id);
            },
            BossEventType::HealthPercentage {
                health_percentage
            } => {
                buffer.write_var_u32(4); // Event type.
                buffer.write_f32_le(health_percentage);
            },
            BossEventType::Title {
                bar_title
            } => {
                buffer.write_var_u32(5); // Event type.
                buffer.write_str(bar_title);
            },
            BossEventType::AppearanceProperties {
                color
            } => {
                buffer.write_var_u32(6); // Event type.
                buffer.write_i16_le(0); // ScreenDarkening is unused.
                buffer.write_var_u32(color as u32);
                buffer.write_var_u32(0); // Overlay is unused.
            },
            BossEventType::Texture {
                color
            } => {
                buffer.write_var_u32(7); // Event type.
                buffer.write_var_u32(color as u32);
                buffer.write_var_u32(0);
            },
            BossEventType::Request {
                player_unique_id
            } => {
                buffer.write_var_u32(8); // Event type.
                buffer.write_var_i64(player_unique_id);
            }
        }

        Ok(())
    }
}