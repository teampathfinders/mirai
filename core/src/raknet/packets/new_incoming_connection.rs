use util::bytes::SharedBuffer;
use util::pyassert;
use util::Result;
use util::{bytes::BinaryRead, Deserialize};

/// Confirms that the connection was successfully initiated.
#[derive(Debug)]
pub struct NewIncomingConnection;

impl NewIncomingConnection {
    /// Unique ID of this packet.
    pub const ID: u8 = 0x13;
}

impl<'a> Deserialize<'a> for NewIncomingConnection {
    fn deserialize<R>(mut reader: R) -> anyhow::Result<Self>
    where
        R: BinaryRead<'a> + 'a,
    {
        pyassert!(reader.read_u8()? == Self::ID);

        // No data in this packet is used, there is no point in decoding it
        Ok(Self)
    }
}
