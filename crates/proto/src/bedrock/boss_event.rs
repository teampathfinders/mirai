use util::{Serialize};
use util::{BinaryWrite};

use crate::bedrock::ConnectedPacket;

/// The boss event colour
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BossEventColor {
    /// Grey
    Grey,
    /// Blue
    Blue,
    /// Red
    Red,
    /// Green
    Green,
    /// Yellow
    Yellow,
    /// Purple
    Purple,
    /// White
    White,
}

/// The boss event type.
#[derive(Debug, Clone)]
pub enum BossEventType<'a> {
    /// Shows a boss event.
    Show {
        /// Title to display above the boss bar.
        bar_title: &'a str,
        /// Colour of the boss event.
        color: BossEventColor,
    },
    /// Registers a player to the boss event.
    RegisterPlayer {
        /// Unique ID of the player.
        player_unique_id: i64
    },
    /// Hides the boss event.
    Hide,
    /// Unregisters a player from the boss event.
    UnregisterPlayer {
        /// Unique ID of the player.
        player_unique_id: i64
    },
    /// Sets the health percentage in the boss bar.
    HealthPercentage {
        /// The health remaining.
        health_percentage: f32
    },
    /// Adds a bossbar.
    Title {
        /// Title to display above the bossbar.
        bar_title: &'a str
    },
    /// Changes the colour of the boss event.
    AppearanceProperties {
        /// New colour.
        color: BossEventColor
    },
    /// Changes the colour of the boss event texture?
    Texture {
        /// New colour
        color: BossEventColor
    },
    /// Not sure what this does.
    Request {
        /// Unique of the player in the request.
        player_unique_id: i64
    },
}

/// Creates a boss event.
#[derive(Debug, Clone)]
pub struct BossEvent<'a> {
    /// Unique ID of the boss.
    pub boss_unique_id: i64,
    /// Event that occurred.
    pub event: BossEventType<'a>,
}

impl ConnectedPacket for BossEvent<'_> {
    const ID: u32 = 0x4a;
}

impl Serialize for BossEvent<'_> {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_var_i64(self.boss_unique_id)?;
        match self.event {
            BossEventType::Show {
                bar_title, color
            } => {
                writer.write_var_u32(0)?; // Event type.

                writer.write_str(bar_title)?;
                writer.write_f32_le(0.0)?; // HealthPercentage is unused.
                writer.write_i16_le(0)?; // ScreenDarkening is unused.
                writer.write_var_u32(color as u32)?;
                writer.write_var_u32(0)?; // Overlay is unused.
            }
            BossEventType::RegisterPlayer {
                player_unique_id
            } => {
                writer.write_var_u32(1)?; // Event type.
                writer.write_var_i64(player_unique_id)?;
            }
            BossEventType::Hide => {
                writer.write_var_u32(2)?; // Event type.
            }
            BossEventType::UnregisterPlayer {
                player_unique_id
            } => {
                writer.write_var_u32(3)?; // Event type.
                writer.write_var_i64(player_unique_id)?;
            }
            BossEventType::HealthPercentage {
                health_percentage
            } => {
                writer.write_var_u32(4)?; // Event type.
                writer.write_f32_le(health_percentage)?;
            }
            BossEventType::Title {
                bar_title
            } => {
                writer.write_var_u32(5)?; // Event type.
                writer.write_str(bar_title)?;
            }
            BossEventType::AppearanceProperties {
                color
            } => {
                writer.write_var_u32(6)?; // Event type.
                writer.write_i16_le(0)?; // ScreenDarkening is unused.
                writer.write_var_u32(color as u32)?;
                writer.write_var_u32(0)?; // Overlay is unused.
            }
            BossEventType::Texture {
                color
            } => {
                writer.write_var_u32(7)?; // Event type.
                writer.write_var_u32(color as u32)?;
                writer.write_var_u32(0)?;
            }
            BossEventType::Request {
                player_unique_id
            } => {
                writer.write_var_u32(8)?; // Event type.
                writer.write_var_i64(player_unique_id)?;
            }
        }

        Ok(())
    }
}