use crate::error::VexResult;
use bytes::BytesMut;

pub trait Encodable {
    fn encode(&self) -> VexResult<BytesMut>;
}

pub trait Decodable {
    fn decode(buffer: BytesMut) -> VexResult<Self>
    where
        Self: Sized;
}