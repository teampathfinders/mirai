use crate::u24::u24;
use crate::{BlockPosition, Vector};
use paste::paste;
use std::io::Write;
use std::net::SocketAddr;
use uuid::Uuid;

use crate::Result;

macro_rules! declare_primitive_fns {
    ($($ty: ident),+) => {
        paste! {$(
            #[doc = concat!("Writes a little endian [`", stringify!($ty), "'] to the writer")]
            #[inline]
            fn [<write_ $ty _le>](&mut self, v: $ty) -> $crate::Result<()> {
                let bytes = v.to_le_bytes();
                self.write_all(&bytes)?;
                Ok(())
            }

            #[doc = concat!("Writes a big endian [`", stringify!($ty), "'] to the writer")]
            #[inline]
            fn [<write_ $ty _be>](&mut self, v: $ty) -> $crate::Result<()> {
                let bytes = v.to_be_bytes();
                self.write_all(&bytes)?;
                Ok(())
            }
        )+}
    }
}

pub trait BinaryWrite: Write {
    declare_primitive_fns!(
        u16, i16, u24, u32, i32, u64, i64, u128, i128, f32, f64
    );

    #[inline]
    fn write_bool(&mut self, v: bool) -> Result<()> {
        self.write_all(&[v as u8])?;
        Ok(())
    }

    #[inline]
    fn write_u8(&mut self, v: u8) -> Result<()> {
        self.write_all(&[v])?;
        Ok(())
    }

    #[inline]
    fn write_i8(&mut self, v: i8) -> Result<()> {
        self.write_all(&[v as u8])?;
        Ok(())
    }

    #[inline]
    fn write_var_u32(&mut self, mut v: u32) -> Result<()> {
        while v >= 0x80 {
            self.write_u8((v as u8) | 0x80)?;
            v >>= 7;
        }
        self.write_u8(v as u8)
    }

    #[inline]
    fn write_var_u64(&mut self, mut v: u64) -> Result<()> {
        while v >= 0x80 {
            self.write_u8((v as u8) | 0x80)?;
            v >>= 7;
        }
        self.write_u8(v as u8)
    }

    #[inline]
    fn write_var_i32(&mut self, v: i32) -> Result<()> {
        let mut ux = (v as u32) << 1;
        if v < 0 {
            ux = !ux;
        }

        self.write_var_u32(ux)
    }

    #[inline]
    fn write_var_i64(&mut self, v: i64) -> Result<()> {
        let mut ux = (v as u64) << 1;
        if v < 0 {
            ux = !ux;
        }

        self.write_var_u64(ux)
    }

    #[inline]
    fn write_str(&mut self, v: &str) -> Result<()> {
        self.write_var_u32(v.len() as u32)?;
        self.write_all(v.as_bytes())?;
        Ok(())
    }

    #[inline]
    fn write_uuid_le(&mut self, v: &Uuid) -> Result<()> {
        let (most, least) = v.as_u64_pair();
        self.write_u64_le(most)?;
        self.write_u64_le(least)
    }

    #[inline]
    fn write_uuid_be(&mut self, v: &Uuid) -> Result<()> {
        let (most, least) = v.as_u64_pair();
        self.write_u64_be(most)?;
        self.write_u64_be(least)
    }

    #[inline]
    fn write_block_pos(&mut self, v: &BlockPosition) -> Result<()> {
        self.write_var_i32(v.x)?;
        self.write_var_u32(v.y)?;
        self.write_var_i32(v.z)
    }

    #[inline]
    fn write_veci<const N: usize>(&mut self, v: &Vector<i32, N>) -> Result<()> {
        for v in v.components() {
            self.write_var_i32(v)?;
        }

        Ok(())
    }

    #[inline]
    fn write_vecf<const N: usize>(&mut self, v: &Vector<f32, N>) -> Result<()> {
        for v in v.components() {
            self.write_f32_le(v)?;
        }

        Ok(())
    }

    fn write_addr(&mut self, v: SocketAddr) -> Result<()> {
        match v {
            SocketAddr::V4(addr_v4) => {
                self.write_u8(4)?;
                self.write_all(addr_v4.ip().octets().as_ref())?;
                self.write_u16_be(v.port())
            }
            SocketAddr::V6(addr_v6) => {
                self.write_u8(6)?;
                self.write_u16_be(23)?; // AF_INET6 family
                self.write_u16_be(v.port())?;
                self.write_u32_be(0)?; // Flow information
                self.write_all(addr_v6.ip().octets().as_ref())?;
                self.write_u32_be(0) // Scope information
            }
        }
    }
}

impl<W: Write> BinaryWrite for W {}
