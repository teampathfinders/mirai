use crate::error::VexResult;
use bytes::BytesMut;

pub trait RaknetPacket {
    const ID: u8;
}

pub trait Encodable {
    fn encode(&self) -> BytesMut;
}

pub trait Decodable {
    fn decode(buf: BytesMut) -> VexResult<Self>
    where
        Self: Sized;
}