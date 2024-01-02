/// The Minecraft dimensions.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum Dimension {
    /// The overworld dimension.
    Overworld,
    /// The nether dimension.
    Nether,
    /// The end dimension.
    End,
}

impl TryFrom<u32> for Dimension {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> anyhow::Result<Self> {
        Ok(match value {
            0 => Self::Overworld,
            1 => Self::Nether,
            2 => Self::End,
            _ => anyhow::bail!("Invalid dimension"),
        })
    }
}