use crate::encodable;

pub struct OpenConnectionReply1 {
    pub server_guid: i64,
    pub security: bool,
    pub mtu: u16,
}

impl OpenConnectionReply1 {
    pub const ID: u8 = 0x06;
}
