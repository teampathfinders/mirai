use bytes::{BufMut, BytesMut};

use crate::error::VexResult;
use crate::network::packets::GamePacket;
use crate::network::traits::Encodable;
use crate::util::WriteExtensions;

#[derive(Debug, Copy, Clone)]
pub enum Status {
    LoginSuccess,
    FailedClient,
    FailedServer,
    PlayerSpawn,
    FailedInvalidTenant,
    FailedVanillaEdu,
    FailedIncompatible,
    FailedServerFull,
    FailedEditorToVanillaMismatch,
    FailedVanillaToEditorMismatch,
}

#[derive(Debug)]
pub struct PlayStatus {
    pub status: Status,
}

impl GamePacket for PlayStatus {
    const ID: u32 = 0x02;
}

impl Encodable for PlayStatus {
    fn encode(&self) -> VexResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(4);

        buffer.put_i32(self.status as i32);

        Ok(buffer)
    }
}
