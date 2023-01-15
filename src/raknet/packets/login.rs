use crate::encodable;

encodable!(
    0x01,
    pub struct OpenConnectionReply1 {
        pub server_guid: i64,
        pub security: bool,
        pub mtu: u16,
    }
);

impl ConnectionRequestAccepted {
    pub const ID: u8 = 0x01;
}
