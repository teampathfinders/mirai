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
use tokio::{task, time, signal};
use tokio_util::sync::CancellationToken;

use crate::error::{VexError, VexResult};
use crate::network::Worker;
use crate::util::AsyncDeque;

const IPV4_LOCAL_ADDR: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
const IPV6_LOCAL_ADDR: Ipv6Addr = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1);

const RECV_BUF_SIZE: usize = 4096;

const INCOMING_QUEUE_SIZE: usize = 25;
const LEAVING_QUEUE_SIZE: usize = 25;

pub const WORKER_COUNT: usize = 1;

#[derive(Debug)]
pub struct RawPacket {
    buffer: BytesMut,
    address: SocketAddr,
}

pub struct NetworkManager {
    flag: CancellationToken,
    ipv4_socket: Arc<UdpSocket>,
    ipv6_socket: Arc<Option<UdpSocket>>,
}

impl NetworkManager {
    pub async fn start(ipv4_port: u16, ipv6_port: Option<u16>) -> VexResult<()> {
        let token = CancellationToken::new();

        // Shutdown on Ctrl-C
        {
            let token = token.clone();
            tokio::spawn(async move {
                signal::ctrl_c().await.unwrap();
                tracing::info!("Ctrl-C detected, token cancelled, shutting down services...");

                token.cancel();
            });
        }

        let ipv4_socket =
            Arc::new(UdpSocket::bind(SocketAddrV4::new(IPV4_LOCAL_ADDR, ipv4_port)).await?);

        let ipv6_socket = Arc::new(if let Some(port) = ipv6_port {
            Some(UdpSocket::bind(SocketAddrV6::new(IPV6_LOCAL_ADDR, port, 0, 0)).await?)
        } else {
            None
        });

        let incoming_queue = Arc::new(AsyncDeque::new(INCOMING_QUEUE_SIZE));
        let leaving_queue = Arc::new(AsyncDeque::new(LEAVING_QUEUE_SIZE));

        let receiver_task = {
            let token = token.clone();
            let ipv4_socket = ipv4_socket.clone();
            let incoming_queue = incoming_queue.clone();

            tokio::spawn(async move {
                Self::v4_receiver_task(token, ipv4_socket, incoming_queue).await;
            })
        };

        let sender_task = {
            let token = token.clone();
            let ipv4_socket = ipv4_socket.clone();
            let leaving_queue = leaving_queue.clone();

            tokio::spawn(async move {
                Self::v4_sender_task(token, ipv4_socket, leaving_queue).await;
            })
        };

        {
            let mut worker_handles = Vec::with_capacity(WORKER_COUNT);
            for _ in 0..WORKER_COUNT {
                worker_handles.push(Worker::new(
                    token.clone(),
                    incoming_queue.clone(),
                    leaving_queue.clone(),
                ));
            }

            // tokio::time::sleep(Duration::from_secs(3)).await;
            // token.cancel();

            for handle in worker_handles {
                let _ = tokio::join!(handle);
            }
        }

        let _ = tokio::join!(receiver_task, sender_task);

        Ok(())
    }

    /// Receives packets from IPv4 clients and adds them to the receive queue
    async fn v4_receiver_task(
        token: CancellationToken,
        socket: Arc<UdpSocket>,
        queue: Arc<AsyncDeque<RawPacket>>,
    ) {
        let mut receive_buffer = [0u8; RECV_BUF_SIZE];

        loop {
            // Wait on both the cancellation token and socket at the same time.
            // The token will immediately take over and stop the task when the server is shutting down.
            let (n, address) = tokio::select! {
                result = socket.recv_from(&mut receive_buffer) => {
                     match result {
                        Ok(r) => r,
                        Err(e) => {
                            tracing::error!("Failed to receive packet: {e:?}");
                            continue;
                        }
                    }
                },
                _ = token.cancelled() => {
                    break
                }
            };

            tracing::debug!("{n:?} bytes from {address:?}");
            queue
                .push(RawPacket {
                    buffer: BytesMut::from(&receive_buffer[..n]),
                    address,
                })
                .await;
        }

        tracing::info!("IPv4 inward service shut down");
    }

    /// Sends packets from the send queue
    async fn v4_sender_task(
        token: CancellationToken,
        socket: Arc<UdpSocket>,
        queue: Arc<AsyncDeque<RawPacket>>,
    ) {
        loop {
            let task = tokio::select! {
                _ = token.cancelled() => break,
                t = queue.pop() => t
            };

            match socket.send_to(&task.buffer, task.address).await {
                Ok(_) => (),
                Err(e) => {
                    tracing::error!("Failed to send packet: {e:?}");
                }
            }
        }

        tracing::info!("IPv4 outward service shut down");
    }
}
