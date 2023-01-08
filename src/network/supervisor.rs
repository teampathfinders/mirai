use std::{net::{Ipv4Addr, SocketAddrV4, Ipv6Addr, SocketAddrV6}, sync::atomic::{Ordering, AtomicBool}};

use tokio::net::UdpSocket;

use crate::error::{VexResult, VexError};

const IPV4_LOCAL_ADDR: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
const IPV6_LOCAL_ADDR: Ipv6Addr = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1);

pub struct NetworkSupervisor {
    ipv4_socket: UdpSocket,
    ipv6_socket: Option<UdpSocket>,

    active_flag: AtomicBool
}

impl NetworkSupervisor {
    pub async fn new(ipv4_port: u16, ipv6_port: Option<u16>) -> VexResult<NetworkSupervisor> {
        let ipv4_socket = UdpSocket::bind(
            SocketAddrV4::new(IPV4_LOCAL_ADDR, ipv4_port)
        ).await?;

        let ipv6_socket = if let Some(port) = ipv6_port {
            Some(UdpSocket::bind(
                SocketAddrV6::new(IPV6_LOCAL_ADDR, port, 0, 0)
            ).await?)
        } else {
            None
        };

        Ok(NetworkSupervisor {
            ipv4_socket,
            ipv6_socket,

            active_flag: AtomicBool::new(true)
        })
    }

    pub async fn start(mut self) -> VexResult<()> {
        while self.is_active() {
            return Ok(())
        }

        Ok(())
    }

    pub fn is_active(&self) -> bool {
        self.active_flag.load(Ordering::Relaxed)
    }
}