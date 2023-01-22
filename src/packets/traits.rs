use bytes::BytesMut;

use crate::error::VexResult;

pub trait GamePacket {
    const ID: u32;
}