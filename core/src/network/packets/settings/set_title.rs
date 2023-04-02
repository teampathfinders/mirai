use util::bytes::{BinaryWrite, MutableBuffer, size_of_varint};
use util::Result;
use util::Serialize;

use crate::network::ConnectedPacket;

/// Title action type.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TitleAction {
    Clear,
    Reset,
    SetTitle,
    SetSubtitle,
    SetActionBar,
    SetDurations,
    TitleTextObject,
    SubtitleTextObject,
    ActionBarTextObject,
}

/// Sets a title for the client.
/// This is basically the same as the /title command in vanilla Minecraft.
#[derive(Debug, Clone)]
pub struct SetTitle<'a> {
    /// Title operation to perform.
    pub action: TitleAction,
    /// Text to display.
    pub text: &'a str,
    /// Fade in duration (in ticks).
    pub fade_in_duration: i32,
    /// How long the title remains on screen (in ticks).
    pub remain_duration: i32,
    /// Fade out duration (in ticks).
    pub fade_out_duration: i32,
    /// XUID of the client.
    pub xuid: &'a str,
    /// Either an uint64 or an empty string.
    pub platform_online_id: &'a str,
}

impl ConnectedPacket for SetTitle<'_> {
    const ID: u32 = 0x58;

    fn serialized_size(&self) -> usize {
        size_of_varint(self.action as i32) +
            size_of_varint(self.text.len() as u32) + self.text.len() +
            size_of_varint(self.fade_in_duration) +
            size_of_varint(self.remain_duration) +
            size_of_varint(self.fade_out_duration) +
            size_of_varint(self.xuid.len() as u32) + self.xuid.len() +
            size_of_varint(self.platform_online_id.len() as u32) + self.platform_online_id.len()
    }
}

impl Serialize for SetTitle<'_> {
    fn serialize(&self, buffer: &mut MutableBuffer) -> anyhow::Result<()> {
        buffer.write_var_i32(self.action as i32)?;
        buffer.write_str(self.text)?;
        buffer.write_var_i32(self.fade_in_duration)?;
        buffer.write_var_i32(self.remain_duration)?;
        buffer.write_var_i32(self.fade_out_duration)?;
        buffer.write_str(self.xuid)?;
        buffer.write_str(self.platform_online_id)
    }
}
