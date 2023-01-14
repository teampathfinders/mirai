use crate::encodable;


encodable!(
     0x01,
    pub struct OpenConnectionReply1{
        server_guid: i64,
        security: bool,
        mtu: u16

    }
);

impl ConnectionRequestAccepted {
    const ID: u8 = 0x01;
}