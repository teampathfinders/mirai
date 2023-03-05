use serde::Serialize;

use crate::{de::Mode, bytes_mut::WriteBuffer, error::Result};

pub struct Serializer {
    mode: Mode,
    buffer: WriteBuffer
}

pub fn to_bytes<T>(value: &T) -> Result<&[u8]> 
where
    T: Serialize
{
    todo!()    
}