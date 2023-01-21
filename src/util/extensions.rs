use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use bytes::{Buf, BufMut};

use crate::error::{VexError, VexResult};

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
        let ip_addr = match ip_type {
            4 => IpAddr::V4(Ipv4Addr::from(self.get_u32())),
            6 => IpAddr::V6(Ipv6Addr::from(self.get_u128())),
            _ => {
                return Err(VexError::InvalidRequest(format!(
                    "Invalid IP type: {}",
                    ip_type
                )))
            }
        };

        let port = self.get_u16();
        Ok(SocketAddr::new(ip_addr, port))
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
pub trait WriteExtensions: BufMut {
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
            }
            SocketAddr::V6(addr_v6) => {
                self.put_u8(6);
                self.put(addr_v6.ip().octets().as_ref());
            }
        }

        self.put_u16(addr.port());
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
