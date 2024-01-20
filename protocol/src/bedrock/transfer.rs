use std::net::SocketAddr;

use util::{BinaryWrite, size_of_varint};
use util::Result;
use util::Serialize;

use crate::bedrock::ConnectedPacket;

/// Transfers the client to another server.
/// The client does this by first returning to the main menu and then connecting to the selected server.
#[derive(Debug, Clone)]
pub struct Transfer<'a> {
    /// Address of the server. This can either be a domain or an IP address.
    pub addr: &'a str,
    /// Port of the server.
    pub port: u16
}

impl<'a> ConnectedPacket for Transfer<'a> {
    const ID: u32 = 0x55;

    fn serialized_size(&self) -> usize {
        let addr_string = self.addr.to_string();

        size_of_varint(addr_string.len() as u32) + addr_string.len() + 2
    }
}

impl<'a> Serialize for Transfer<'a> {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_str(self.addr)?;
        writer.write_u16_le(self.port)
    }
}
