use crate::error::{VexError, VexResult};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::io::Read;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

pub trait ReadAddress: Buf {
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
}

pub trait WriteAddress: BufMut {
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
}

impl<T: Buf> ReadAddress for T {}
impl<T: BufMut> WriteAddress for T {}
