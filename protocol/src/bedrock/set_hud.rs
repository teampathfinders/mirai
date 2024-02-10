use util::{BinaryWrite, Serialize};

use super::ConnectedPacket;

/// The element to change in the HUD.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum HudElement {
    /// The small doll displayed in the top left corner when doing things such as sneaking.
    PaperDoll,
    /// The armour points display.
    Armor,
    /// Text shown above your hotbar when you switch to a different item.
    Tooltips,
    /// Touch controls on touch devices.
    TouchControls,
    /// The crosshair on screen.
    Crosshair,
    /// The hotbar.
    Hotbar,
    /// Health (only shown in survival)
    Health,
    /// Experience progress bar.
    ProgressBar,
    /// The hunger bar.
    Hunger,
    /// Air bubbles in case the player is underwater.
    AirBubbles,
    /// Health of the horse that is being ridden.
    HorseHealth
}

/// Visibility of a HUD element.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum HudVisibility {
    /// Hides the given HUD elements.
    Hide,
    /// Makes the given HUD elements visible again.
    Reset
}

/// Hides or shows HUD elements.
#[derive(Debug)]
pub struct SetHud<'a> {
    /// Elements to change.
    pub elements: &'a [HudElement],
    /// New visibility of the given elements.
    pub visibibility: HudVisibility
}

impl ConnectedPacket for SetHud<'_> {
    const ID: u32 = 0x134;
}

impl Serialize for SetHud<'_> {
    fn size_hint(&self) -> Option<usize> {
        let hint = 1 + 1 + self.elements.len() + 1;

        Some(hint)
    }

    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_var_u32(self.elements.len() as u32)?;
        self.elements.iter().try_for_each(|elem| writer.write_u8(*elem as u8))?;
        writer.write_u8(self.visibibility as u8)
    }
}
