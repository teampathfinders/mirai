use serde::{ser, Serialize};
use std::io::Write;
use std::marker::PhantomData;
use util::bytes::MutableBuffer;
use util::{Error, Result};

use crate::{de, BigEndian, Flavor, LittleEndian, NbtFlavor};

#[inline]
pub fn to_bytes<T, F>(t: &T) -> Result<MutableBuffer>
where
    T: Serialize,
    F: NbtFlavor,
{
    let mut ser = Serializer::<MutableBuffer, F>::new(MutableBuffer::new());
    // t.serialize(&mut ser)?;
    Ok(ser.into_inner())
}

#[derive(Debug)]
pub struct Serializer<W, E>
where
    W: Write,
    E: NbtFlavor,
{
    writer: W,
    _marker: PhantomData<E>,
}

impl<W, E> Serializer<W, E>
where
    W: Write,
    E: NbtFlavor,
{
    #[inline]
    pub fn new(w: W) -> Serializer<W, E> {
        Serializer { writer: w, _marker: PhantomData }
    }

    #[inline]
    pub fn into_inner(self) -> W {
        self.writer
    }
}

// impl<'a, W, E> ser::Serializer for &'a mut Serializer<W, E> {
//     type Ok = ();
//     type Error = Error;
//
//     type SerializeSeq = Self;
//     type SerializeTuple = Self;
//     type SerializeTupleStruct = Self;
//     type SerializeTupleVariant = Self;
//     type SerializeMap = Self;
//     type SerializeStruct = Self;
//     type SerializeStructVariant = Self;
//
//     #[inline]
//     fn serialize_bool(self, v: bool) -> Result<()> {
//         self.writer.write_bool(v);
//         Ok(())
//     }
//
//     // ...
// }
