
use util::bail;
use util::{BinaryRead};
use util::Deserialize;

use crate::bedrock::ConnectedPacket;

/// The type of violation.
#[derive(Debug, Copy, Clone)]
pub enum ViolationType {
    /// The server sent a malformed packet.
    Malformed,
}

impl TryFrom<i32> for ViolationType {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Malformed,
            _ => bail!(Malformed, "Invalid violation type {}", value),
        })
    }
}

/// Severity of the violation.
#[derive(Debug, Copy, Clone)]
pub enum ViolationSeverity {
    /// First warning given by the client.
    Warning,
    /// Final warning before the client will disconnect.
    FinalWarning,
    /// Client has disconnect from the server.
    TerminatingConnection,
}

impl TryFrom<i32> for ViolationSeverity {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Warning,
            1 => Self::FinalWarning,
            2 => Self::TerminatingConnection,
            _ => bail!(Malformed, "Invalid violation severity {}", value),
        })
    }
}

/// (Sometimes) sent by the client when the server sends a broken packet.
/// This packet is pretty useless since the client almost never actually sends it.
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
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let warning_type = ViolationType::try_from(reader.read_var_i32()?)?;
        let severity = ViolationSeverity::try_from(reader.read_var_i32()?)?;
        let packet_id = reader.read_var_i32()?;
        let context = reader.read_str()?;

        Ok(Self {
            warning_type,
            severity,
            packet_id,
            context,
        })
    }
}
