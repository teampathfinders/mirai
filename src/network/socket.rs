use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::net::UdpSocket;
use crate::error::VexResult;

const V4_LOCAL_ADDR: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);

pub struct RawSocket {
    handle: UdpSocket
}

impl RawSocket {
    pub async fn new(port: u16) -> VexResult<RawSocket> {
        let handle = UdpSocket::bind(SocketAddrV4::new(V4_LOCAL_ADDR, port)).await?;

        Ok(RawSocket {
            handle
        })
    }
}