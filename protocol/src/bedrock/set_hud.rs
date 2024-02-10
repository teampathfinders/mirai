use util::{BinaryWrite, Serialize};

use super::ConnectedPacket;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum HudElement {
    PaperDoll,
    Armor,
    Tooltips,
    TouchControls,
    Crosshair,
    Hotbar,
    Health,
    ProgressBar,
    Hunger,
    AirBubbles,
    HorseHealth
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum HudVisibility {
    Hide,
    Reset
}

#[derive(Debug)]
pub struct SetHud<'a> {
    pub elements: &'a [HudElement],
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
