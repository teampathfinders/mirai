use bytes::BytesMut;

use vex_common::{bail, Decodable, ReadExtensions, VError, VResult};

use crate::bail;
use crate::network::Decodable;
use crate::network::packets::GamePacket;
use crate::util::ReadExtensions;

#[derive(Debug, Copy, Clone)]
pub enum ViolationType {
    Malformed
}

impl TryFrom<u32> for ViolationType {
    type Error = VError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Malformed,
            _ => bail!(BadPacket, "Invalid violation type {}", value)
        })
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ViolationSeverity {
    Warning,
    FinalWarning,
    TerminatingConnection,
}

impl TryFrom<u32> for ViolationSeverity {
    type Error = VError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Warning,
            1 => Self::FinalWarning,
            2 => Self::TerminatingConnection,
            _ => bail!(BadPacket, "Invalid violation severity {}", value)
        })
    }
}

#[derive(Debug)]
pub struct ViolationWarning {
    /// Type of the violation.
    pub warning_type: ViolationType,
    /// Severity of the violation.
    pub severity: ViolationSeverity,
    /// ID of the invalid packet.
    pub packet_id: u32,
    /// Description of the violation.
    pub context: String,
}

impl GamePacket for ViolationWarning {
    const ID: u32 = 0x9c;
}

impl Decodable for ViolationWarning {
    fn decode(mut buffer: BytesMut) -> VResult<Self> {
        tracing::debug!("{:x?}", buffer.as_ref());

        let warning_type = ViolationType::try_from(buffer.get_var_u32()?)?;
        let severity = ViolationSeverity::try_from(buffer.get_var_u32()?)?;
        let packet_id = buffer.get_var_u32()?;
        let context = buffer.get_string()?;

        Ok(Self {
            warning_type,
            severity,
            packet_id,
            context,
        })
    }
}