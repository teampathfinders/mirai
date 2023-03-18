use std::io::Write;
use std::marker::PhantomData;
use serde::Serialize;
use util::bytes::MutableBuffer;
use util::Result;

use crate::{BigEndian, Flavor, LittleEndian, NbtFlavor};

#[derive(Debug)]
pub struct Serializer<W, E>
where
    W: Write,
    E: NbtFlavor,
{
    writer: W,
    _marker: PhantomData<E>
}

impl<W, E> Serializer<W, E>
where
    W: Write, E: NbtFlavor,
{
    #[inline]
    pub fn into_inner(self) -> W {
        self.writer
    }
}

impl<W> Serializer<W, BigEndian>
where
    W: Write,
{
    #[inline]
    pub fn new(w: W) -> Serializer<W, BigEndian> {
        Serializer {
            writer: w,
            _marker: PhantomData
        }
    }

    pub fn to_bytes(self) -> Result<()> {
        todo!();
    }
}

impl<W> Serializer<W, LittleEndian>
where
    W: Write,
{
    #[inline]
    pub fn new(w: W) -> Serializer<W, LittleEndian> {
        Serializer {
            writer: w,
            _marker: PhantomData
        }
    }

    pub fn to_bytes(self) -> Result<()> {
        todo!();
    }
}