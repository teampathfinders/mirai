use std::fmt::Debug;
use std::io::Write;
use std::ops::Deref;
use std::{fmt, io};
use crate::bytes::BufMut;

pub struct WriteBuffer(Vec<u8>);

impl WriteBuffer {
    #[inline]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }
}

impl From<Vec<u8>> for WriteBuffer {
    #[inline]
    fn from(b: Vec<u8>) -> Self {
        Self(b)
    }
}

impl Debug for WriteBuffer {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self.0)
    }
}

impl Deref for WriteBuffer {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl Write for WriteBuffer {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.extend_from_slice(buf);
        Ok(buf.len())
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl BufMut for WriteBuffer {
    #[inline]
    fn write_bool(&mut self, value: bool) {
        self.write_u8(value as u8);
    }

    #[inline]
    fn write_u8(&mut self, value: u8) {
        self.0.push(value);
    }

    #[inline]
    fn write_u16(&mut self, value: u16) {
        self.0.extend(value.to_be_bytes());
    }

    #[inline]
    fn write_u32(&mut self, value: u32) {
        self.0.extend(value.to_be_bytes());
    }

    #[inline]
    fn write_u64(&mut self, value: u64) {
        self.0.extend(value.to_be_bytes());
    }

    #[inline]
    fn write_u128(&mut self, value: u128) {
        self.0.extend(value.to_be_bytes());
    }

    #[inline]
    fn write_i8(&mut self, value: i8) {
        self.0.push(value as u8);
    }

    #[inline]
    fn write_i16(&mut self, value: i16) {
        self.0.extend(value.to_be_bytes());
    }

    #[inline]
    fn write_i32(&mut self, value: i32) {
        self.0.extend(value.to_be_bytes());
    }

    #[inline]
    fn write_i64(&mut self, value: i64) {
        self.0.extend(value.to_be_bytes());
    }

    #[inline]
    fn write_i128(&mut self, value: i128) {
        self.0.extend(value.to_be_bytes());
    }

    #[inline]
    fn write_u16_le(&mut self, value: u16) {
        self.0.extend(value.to_le_bytes());
    }

    #[inline]
    fn write_u32_le(&mut self, value: u32) {
        self.0.extend(value.to_le_bytes());
    }

    #[inline]
    fn write_u64_le(&mut self, value: u64) {
        self.0.extend(value.to_le_bytes());
    }

    #[inline]
    fn write_u128_le(&mut self, value: u128) {
        self.0.extend(value.to_le_bytes());
    }

    #[inline]
    fn write_i16_le(&mut self, value: i16) {
        self.0.extend(value.to_le_bytes());
    }

    #[inline]
    fn write_i32_le(&mut self, value: i32) {
        self.0.extend(value.to_le_bytes());
    }

    #[inline]
    fn write_i64_le(&mut self, value: i64) {
        self.0.extend(value.to_le_bytes());
    }

    #[inline]
    fn write_i128_le(&mut self, value: i128) {
        self.0.extend(value.to_le_bytes());
    }

    #[inline]
    fn write_f32(&mut self, value: f32) {
        self.0.extend(value.to_be_bytes());
    }

    #[inline]
    fn write_f32_le(&mut self, value: f32) {
        self.0.extend(value.to_le_bytes());
    }

    #[inline]
    fn write_f64(&mut self, value: f64) {
        self.0.extend(value.to_be_bytes());
    }

    #[inline]
    fn write_f64_le(&mut self, value: f64) {
        self.0.extend(value.to_le_bytes());
    }
}
