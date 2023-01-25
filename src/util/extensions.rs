use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4};

use bytes::{Buf, BufMut};
use lazy_static::lazy_static;

use crate::{error, vex_assert};
use crate::error::{VexError, VexResult};

pub const IPV4_MEM_SIZE: usize = 1 + 4 + 2;
pub const IPV6_MEM_SIZE: usize = 1 + 2 + 2 + 4 + 16 + 4;

lazy_static! {
    pub static ref EMPTY_IPV4_ADDRESS: SocketAddr =
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(255, 255, 255, 255), 19132));
}

/// Provides extra functions for byte buffers.
/// This trait implements read functions for exotic formats and
/// IP addresses that the default [`Bytes`](bytes::Bytes) implementation does not provide.
pub trait ReadExtensions: Buf {
    /// Reads an IP address from a buffer.
    /// Format:
    ///
    /// * One byte for IP type (4 or 6),
    /// * If IPv4, 4 bytes for the 4 octets,
    /// * If IPv6, 16 bytes for the 4 octets,
    /// * Unsigned short for port.
    ///
    /// This method fails if the IP type is a value other than 4 or 6.
    fn get_addr(&mut self) -> VexResult<SocketAddr> {
        let ip_type = self.get_u8();
        Ok(match ip_type {
            4 => {
                let addr = IpAddr::V4(Ipv4Addr::from(self.get_u32()));
                let port = self.get_u16();

                SocketAddr::new(addr, port)
            }
            6 => {
                self.advance(2); // IP family (AF_INET6)
                let port = self.get_u16();
                self.advance(4); // Flow information
                let addr = IpAddr::V6(Ipv6Addr::from(self.get_u128()));
                self.advance(4); // Scope ID

                SocketAddr::new(addr, port)
            }
            _ => {
                return Err(VexError::InvalidRequest(format!(
                    "Invalid IP type: {}",
                    ip_type
                )))
            }
        })
    }

    /// Reads a boolean from the stream.
    fn get_bool(&mut self) -> bool {
        self.get_u8() == 1
    }

    /// Reads a UTF-8 string prefixed with a varint specifying its length.
    /// This should be used for strings encoded by Minecraft.
    ///
    /// It can fail if the varint could not be read correctly.
    ///
    /// See [`get_raknet_string`] for an alternative for Raknet.
    fn get_string(&mut self) -> VexResult<String> {
        let length = self.get_var_u32()? as usize;
        let buffer = &self.chunk()[..length];
        let string = String::from_utf8_lossy(buffer).to_string();

        self.advance(length);
        Ok(string)
    }

    /// Reads a UTF-8 string encoded with an unsigned short in front, specifying its length.
    /// This should be used for strings encoded by Raknet.
    ///
    /// See [`get_string`] for an alternative for Minecraft.
    fn get_raknet_string(&mut self) -> String {
        let length = self.get_u16() as usize;
        let buffer = &self.chunk()[..length];
        let string = String::from_utf8_lossy(buffer).to_string();

        self.advance(length);
        string
    }

    /// Reads a variable size unsigned big endian 32-bit integer from the stream.
    fn get_var_u32(&mut self) -> VexResult<u32> {
        let mut v = 0;
        let mut i = 0;
        while i < 35 {
            let b = self.get_u8();
            v |= ((b & 0x7f) as u32) << i;
            if b & 0x80 == 0 {
                return Ok(v);
            }
            i += 7;
        }

        Err(error!(
            InvalidRequest,
            "Variable 32-bit integer did not end after 5 bytes"
        ))
    }

    /// Reads a 24-bit unsigned little-endian integer from the buffer.
    fn get_u24_le(&mut self) -> u32 {
        let a = self.get_u8() as u32;
        let b = self.get_u8() as u32;
        let c = self.get_u8() as u32;

        a | (b << 8) | (c << 16)
    }
}

/// Provides extra functions for byte buffers.
/// This trait implements write functions for exotic formats and
/// IP addresses that the default [`BytesMut`](bytes::BytesMut) implementation does not provide.
pub trait WriteExtensions: BufMut + Sized {
    /// Writes an IP address into a buffer.
    ///
    /// IP format described in [`get_addr`](ReadExtensions::get_addr).
    fn put_addr(&mut self, addr: SocketAddr)
        where
            Self: Sized,
    {
        match addr {
            SocketAddr::V4(addr_v4) => {
                self.put_u8(4);
                self.put(addr_v4.ip().octets().as_ref());
                self.put_u16(addr.port());
            }
            SocketAddr::V6(addr_v6) => {
                self.put_u8(6);
                self.put_u16(23); // AF_INET6 family
                self.put_u16(addr.port());
                self.put_u32(0); // Flow information
                self.put(addr_v6.ip().octets().as_ref());
                self.put_u32(0); // Scope information
            }
        }
    }

    /// Writes a boolean to the stream.
    fn put_bool(&mut self, value: bool) {
        self.put_u8(value as u8);
    }

    /// Writes a varint-prefixed string to the stream.
    /// This should be used for strings designated for Minecraft.
    ///
    /// See [`put_raknet_string`] for an alternative for Raknet.
    fn put_string(&mut self, value: &str) {
        self.put_var_u32(value.len() as u32);
        self.put(value.as_bytes());
    }

    /// Writes a short-prefixed string to the stream.
    /// This should be used for strings designated for Raknet.
    ///
    /// See [`put_string`] for an alternative for Minecraft.
    fn put_raknet_string(&mut self, value: &str) {
        self.put_u16(value.len() as u16);
        self.put(value.as_bytes());
    }

    /// Writes a variable size unsigned big endian 32-bit integer to the stream.
    fn put_var_u32(&mut self, mut value: u32) {
        while value >= 0x80 {
            self.put_u8(((value) as u8) | 0x80);
            value >>= 7;
        }
        self.put_u8(value as u8);
    }

    /// Writes a 24-bit unsigned little-endian integer to the buffer.
    fn put_u24_le(&mut self, value: u32) {
        assert!(value < 2u32.pow(24));

        let a = value & 0xff;
        let b = (value >> 8) & 0xff;
        let c = (value >> 16) & 0xff;

        self.put_u8(a as u8);
        self.put_u8(b as u8);
        self.put_u8(c as u8);
    }
}

/// Implement [`ReadExtensions`] for all types that implement [`Buf`].
impl<T: Buf> ReadExtensions for T {}
/// Implement [`WriteExtensions`] for all types that implement [`BufMut`].
impl<T: BufMut> WriteExtensions for T {}
