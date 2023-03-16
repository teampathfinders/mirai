use crate::ConnectedPacket;
use util::bail;
use util::Deserialize;
use util::{Error, Result};
use util::bytes::{BinaryReader, SharedBuffer};

#[derive(Debug, Copy, Clone)]
pub enum ViolationType {
    Malformed,
}

impl TryFrom<i32> for ViolationType {
    type Error = Error;

    fn try_from(value: i32) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Malformed,
            _ => bail!(Malformed, "Invalid violation type {}", value),
        })
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ViolationSeverity {
    Warning,
    FinalWarning,
    TerminatingConnection,
}

impl TryFrom<i32> for ViolationSeverity {
    type Error = Error;

    fn try_from(value: i32) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Warning,
            1 => Self::FinalWarning,
            2 => Self::TerminatingConnection,
            _ => bail!(Malformed, "Invalid violation severity {}", value),
        })
    }
}

#[derive(Debug)]
pub struct ViolationWarning<'a> {
    /// Type of the violation.
    pub warning_type: ViolationType,
    /// Severity of the violation.
    pub severity: ViolationSeverity,
    /// ID of the invalid packet.
    pub packet_id: i32,
    /// Description of the violation.
    pub context: &'a str,
}

impl<'a> ConnectedPacket for ViolationWarning<'a> {
    const ID: u32 = 0x9c;
}

impl<'a> Deserialize<'a> for ViolationWarning<'a> {
    fn deserialize(mut buffer: SharedBuffer<'a>) -> Result<Self> {
        let warning_type = ViolationType::try_from(buffer.read_var_i32()?)?;
        let severity = ViolationSeverity::try_from(buffer.read_var_i32()?)?;
        let packet_id = buffer.read_var_i32()?;
        let context = buffer.read_str()?;

        Ok(Self {
            warning_type,
            severity,
            packet_id,
            context,
        })
    }
}
