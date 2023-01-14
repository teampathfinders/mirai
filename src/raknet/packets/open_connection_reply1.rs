use crate::encodable;

encodable!(
    0x06,
    pub struct OpenConnectionReply1 {
        server_guid: i64,
        security: bool,
        mtu: u16
    }
);

