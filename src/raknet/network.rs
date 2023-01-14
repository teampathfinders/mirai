use bytes::BytesMut;
use std::mem::MaybeUninit;
use std::net::SocketAddr;
use std::time::Duration;
use std::{
    net::{Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use tokio::net::UdpSocket;
use tokio::{signal, task, time};
use tokio_util::sync::CancellationToken;
use crate::data::ServerData;

use crate::error::{VexError, VexResult};
use crate::raknet::packets::{Decodable, RaknetPacket, RawPacket, UnconnectedPing, UnconnectedPong};
use crate::util::AsyncDeque;
use crate::worker::Worker;

const IPV4_LOCAL_ADDR: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
const IPV6_LOCAL_ADDR: Ipv6Addr = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1);

const RECV_BUF_SIZE: usize = 4096;

const INCOMING_QUEUE_SIZE: usize = 25;
const LEAVING_QUEUE_SIZE: usize = 25;

pub const WORKER_COUNT: usize = 1;

pub struct NetController {
    data: Arc<ServerData>,
    global_token: CancellationToken,
    ipv4_socket: Arc<UdpSocket>,
    ipv4_port: u16,

    inward_queue: Arc<AsyncDeque<RawPacket>>,
    outward_queue: Arc<AsyncDeque<RawPacket>>

    // ipv6_socket: Arc<Option<UdpSocket>>,
    // ipv6_port: Option<u16>
}

impl NetController {
    pub async fn new(
        data: Arc<ServerData>,
        global_token: CancellationToken,
        ipv4_port: u16
    ) -> VexResult<NetController> {
        let ipv4_socket =
            Arc::new(UdpSocket::bind(SocketAddrV4::new(IPV4_LOCAL_ADDR, ipv4_port)).await?);
        tracing::info!("Set up IPv4 socket on port {ipv4_port}");

        Ok(NetController {
            data,
            global_token,
            ipv4_socket,
            ipv4_port,

            inward_queue: Arc::new(AsyncDeque::new(10)),
            outward_queue: Arc::new(AsyncDeque::new(10))
        })
    }

    pub fn ipv4_port(&self) -> u16 {
        self.ipv4_port
    }

    pub async fn start(self: Arc<Self>) -> VexResult<()> {
        // let ipv6_socket = Arc::new(if let Some(port) = ipv6_port {
        //     Some(UdpSocket::bind(SocketAddrV6::new(IPV6_LOCAL_ADDR, port, 0, 0)).await?)
        // } else {
        //     None
        // });

        let receiver_task = {
            let controller = self.clone();
            tokio::spawn(async move {
                controller.v4_receiver_task().await
            })
        };

        let sender_task = {
            tokio::spawn(async move {
                self.v4_sender_task().await
            })
        };

        let _ = tokio::join!(receiver_task, sender_task);
        Ok(())
    }

    async fn handle_offline_packet(self: Arc<Self>, packet: RawPacket) -> VexResult<()> {
        let id = packet.packet_id().ok_or(VexError::InvalidRequest("Packet is empty".to_string()))?;
        tracing::info!("{id}");

        match id {
            UnconnectedPing::ID => self.handle_unconnected_ping(packet).await?,
            _ => todo!("Packet type not implemented")
        }

        Ok(())
    }

    async fn handle_unconnected_ping(self: Arc<Self>, packet: RawPacket) -> VexResult<()> {
        let ping = UnconnectedPing::decode(packet.buffer.clone())?;
        let pong = UnconnectedPong::build()
            .time(*ping.time())
            .server_guid(self.data.guid())
            .metadata(self.data.metadata()?)
            .encode();

        self.ipv4_socket.send_to(pong.as_ref(), packet.address).await?;
        Ok(())
    }

    /// Receives packets from IPv4 clients and adds them to the receive queue
    async fn v4_receiver_task(self: Arc<Self>) {
        tracing::info!("Inward v4 service online");

        let mut receive_buffer = [0u8; RECV_BUF_SIZE];

        loop {
            // Wait on both the cancellation token and socket at the same time.
            // The token will immediately take over and stop the task when the server is shutting down.
            let (n, address) = tokio::select! {
                result = self.ipv4_socket.recv_from(&mut receive_buffer) => {
                     match result {
                        Ok(r) => r,
                        Err(e) => {
                            tracing::error!("Failed to receive packet: {e:?}");
                            continue;
                        }
                    }
                },
                _ = self.global_token.cancelled() => {
                    break
                }
            };

            let mut raw_packet = RawPacket {
                buffer: BytesMut::from(&receive_buffer[..n]),
                address
            };

            if raw_packet.is_offline_packet() {
                let controller = self.clone();
                tokio::spawn(async move {
                    controller.handle_offline_packet(raw_packet).await;
                });
            } else {
                todo!("Send packet to session");
            }
        }

        tracing::info!("Inward v4 service shut down");
    }

    /// Sends packets from the send queue
    async fn v4_sender_task(self: Arc<Self>) {
        tracing::info!("Outward v4 service online");

        loop {
            let task = tokio::select! {
                _ = self.global_token.cancelled() => break,
                t = self.outward_queue.pop() => t
            };

            match self.ipv4_socket.send_to(&task.buffer, task.address).await {
                Ok(_) => (),
                Err(e) => {
                    tracing::error!("Failed to send packet: {e:?}");
                }
            }
        }

        tracing::info!("Outward v4 service shut down");
    }
}
