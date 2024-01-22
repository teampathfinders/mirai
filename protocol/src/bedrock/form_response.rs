use util::{BinaryRead};
use util::Deserialize;
use crate::bedrock::ConnectedPacket;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CancelReason {
    /// The client closed the form.
    Closed,
    /// The client was busy. This for example happens when the client's chat is open and the form cannot be displayed.
    Busy
}

impl TryFrom<u8> for CancelReason {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> anyhow::Result<CancelReason> {
        Ok(match value {
            0 => CancelReason::Closed,
            1 => CancelReason::Busy,
            v => anyhow::bail!("Expected either 0 or 1 for forms cancel reason, got {v}")
        })
    }
}

#[derive(Debug)]
pub struct FormResponseData<'a> {
    pub id: u32,
    pub response_data: Option<&'a str>,
    pub cancel_reason: Option<CancelReason>
}

impl<'a> ConnectedPacket for FormResponseData<'a> {
    const ID: u32 = 0x65;
}

impl<'a> Deserialize<'a> for FormResponseData<'a> {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<FormResponseData<'a>> {
        let id = reader.read_var_u32()?;
        let has_data = reader.read_bool()?;
        let response_data = if has_data {
            Some(reader.read_str()?)
        } else {
            None
        };

        let has_reason = reader.read_bool()?;
        let cancel_reason = if has_reason {
            Some(reader.read_u8()?)
        } else {
            None
        };

        Ok(FormResponseData {
            id,
            response_data,
            cancel_reason: cancel_reason.map(CancelReason::try_from).transpose()?
        })
    }
}