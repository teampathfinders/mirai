use bytes::BytesMut;

#[derive(Debug)]
pub struct Frame {
    sequence_number: u32,
    body: BytesMut,
}
