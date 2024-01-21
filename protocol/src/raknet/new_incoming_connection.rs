use util::iassert;
use util::{BinaryRead, Deserialize};

/// Confirms that the connection was successfully initiated.
#[derive(Debug)]
pub struct NewIncomingConnection;

impl NewIncomingConnection {
    /// Unique ID of this packet.
    pub const ID: u8 = 0x13;
}

impl<'a> Deserialize<'a> for NewIncomingConnection {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        iassert!(reader.read_u8()? == Self::ID);

        // No data in this packet is used, there is no point in decoding it
        Ok(Self)
    }
}
