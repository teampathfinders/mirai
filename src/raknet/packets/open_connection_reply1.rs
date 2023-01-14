use crate::encodable;

pub struct OpenConnectionReply1 {
    server_guid: i64,
    security: bool,
    mtu: u16,
}

impl OpenConnectionReply1 {
    const ID: u8 = 0x06;
}
