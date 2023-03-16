use crate::{bail, BlockPosition, Result};
use crate::{u24::u24, Vector};
use paste::paste;
use std::mem;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use uuid::Uuid;

macro_rules! declare_primitive_fns {
    ($($ty: ident),+) => {
        paste! {$(
            #[doc = concat!("Reads a little endian [`", stringify!($ty), "`] from the reader")]
            #[inline]
            fn [<read_ $ty _le>] (&mut self) -> $crate::Result<$ty> {
                let bytes = self.take_const()?;
                Ok(<$ty>::from_le_bytes(bytes))
            }

            #[doc = concat!("Reads a big endian [`", stringify!($ty), "`] from the reader")]
            #[inline]
            fn [<read_ $ty _be>] (&mut self) -> $crate::Result<$ty> {
                let bytes = self.take_const()?;
                Ok(<$ty>::from_be_bytes(bytes))
            }
        )+}
    }
}

/// Adds binary reading capabilities to a reader.
pub trait BinaryReader<'a> {
    declare_primitive_fns!(u16, i16, u24, u32, i32, u64, i64, u128, i128, f32, f64);

    fn advance(&mut self, n: usize) -> Result<()>;
    /// Takes `n` bytes out of the reader.
    fn take_n(&mut self, n: usize) -> Result<&'a [u8]>;
    /// Takes `N` bytes out of the reader.
    /// This can be used to get sized arrays if the size is known at compile time.
    fn take_const<const N: usize>(&mut self) -> Result<[u8; N]>;
    /// Takes `n` bytes out of the reader without advancing the cursor.
    fn peek(&self, n: usize) -> Result<&[u8]>;
    /// Takes `N` bytes out of the reader without advancing the cursor.
    /// /// This can be used to get sized arrays if the size is known at compile time.
    fn peek_const<const N: usize>(&self) -> Result<[u8; N]>;

    /// Reads a [`bool`] from the reader.
    #[inline]
    fn read_bool(&mut self) -> Result<bool> {
        Ok(self.take_const::<1>()?[0] != 0)
    }

    /// Reads a [`u8`] from the reader.
    #[inline]
    fn read_u8(&mut self) -> Result<u8> {
        Ok(self.take_const::<1>()?[0])
    }

    /// Reads an [`i8`] from the reader.
    #[inline]
    fn read_i8(&mut self) -> Result<i8> {
        Ok(self.take_const::<1>()?[0] as i8)
    }

    /// Reads a variable size [`u32`] from the reader.
    #[inline]
    fn read_var_u32(&mut self) -> Result<u32> {
        let mut v = 0;
        let mut i = 0;
        while i < 35 {
            let b = self.read_u8()?;
            v |= ((b & 0x7f) as u32) << i;
            if b & 0x80 == 0 {
                return Ok(v);
            }
            i += 7;
        }

        bail!(
            Malformed,
            "variable 32-bit integer did not end after 5 bytes"
        )
    }

    /// Reads a variable size [`i32`] from the reader.
    #[inline]
    fn read_var_u64(&mut self) -> Result<u64> {
        let mut v = 0;
        let mut i = 0;
        while i < 70 {
            let b = self.read_u8()?;
            v |= ((b & 0x7f) as u64) << i;

            if b & 0x80 == 0 {
                return Ok(v);
            }

            i += 7;
        }

        bail!(
            Malformed,
            "variable 64-bit integer did not end after 10 bytes"
        )
    }

    /// Reads a variable size [`u64`] from the reader.
    #[inline]
    fn read_var_i32(&mut self) -> Result<i32> {
        let vx = self.read_var_u32()?;
        let mut v = (vx >> 1) as i32;

        if vx & 1 != 0 {
            v = !v;
        }

        Ok(v)
    }

    /// Reads a variable size [`i64`] from the reader.
    #[inline]
    fn read_var_i64(&mut self) -> Result<i64> {
        let vx = self.read_var_u64()?;
        let mut v = (vx >> 1) as i64;

        if vx & 1 != 0 {
            v = !v;
        }

        Ok(v)
    }

    /// Reads a string prefixed by a variable u32.
    #[inline]
    fn read_str(&mut self) -> Result<&'a str> {
        let len = self.read_var_u32()?;
        let data = self.take_n(len as usize)?;

        Ok(std::str::from_utf8(data)?)
    }

    #[inline]
    fn read_block_pos(&mut self) -> Result<BlockPosition> {
        let x = self.read_var_i32()?;
        let y = self.read_var_u32()?;
        let z = self.read_var_i32()?;

        Ok(BlockPosition::new(x, y, z))
    }

    #[inline]
    fn read_veci<const N: usize>(&mut self) -> Result<Vector<i32, N>> {
        let mut x = [0; N];
        for i in 0..N {
            x[i] = self.read_var_i32()?;
        }
        Ok(Vector::from(x))
    }

    #[inline]
    fn read_vecf<const N: usize>(&mut self) -> Result<Vector<f32, N>> {
        let mut x = [0.0; N];
        for i in 0..N {
            x[i] = self.read_f32_le()?;
        }
        Ok(Vector::from(x))
    }

    fn read_addr(&mut self) -> Result<SocketAddr> {
        let variant = self.read_u8()?;
        Ok(match variant {
            4 => {
                let addr = IpAddr::V4(Ipv4Addr::from(self.read_u32_be()?));
                let port = self.read_u16_be()?;

                SocketAddr::new(addr, port)
            }
            6 => {
                self.advance(2)?; // IP family (AF_INET6)
                let port = self.read_u16_be()?;
                self.advance(4)?; // Flow information
                let addr = IpAddr::V6(Ipv6Addr::from(self.read_u128_be()?));
                self.advance(4)?; // Scope ID

                SocketAddr::new(addr, port)
            }
            _ => {
                bail!(
                    Malformed,
                    "Invalid IP type {variant}, expected either 4 or 6"
                );
            }
        })
    }
}
