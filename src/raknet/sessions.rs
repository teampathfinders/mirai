use crate::error::VexResult;
use crate::raknet::packet::RawPacket;
use crate::util::AsyncDeque;
use crate::vex_error;
use bytes::BytesMut;
use dashmap::DashMap;
use parking_lot::RwLock;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;
use crate::raknet::{CompoundCollector, Frame, FrameSet};
use crate::raknet::packets::{Ack, Decodable, Nack};

const INTERNAL_TICK_INTERVAL: Duration = Duration::from_millis(1000 / 20);
const TICK_INTERVAL: Duration = Duration::from_millis(1000 / 20);
const SESSION_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug)]
pub struct Session {
    address: SocketAddr,
    guid: i64,

    last_update: RwLock<Instant>,
    active: CancellationToken,

    compound_collector: CompoundCollector,
    queue: AsyncDeque<BytesMut>,
}

impl Session {
    pub fn new(address: SocketAddr, client_guid: i64) -> Arc<Self> {
        let session = Arc::new(Self {
            address,
            guid: client_guid,
            last_update: RwLock::new(Instant::now()),
            active: CancellationToken::new(),
            compound_collector: CompoundCollector::new(),
            queue: AsyncDeque::new(5),
        });

        // Session ticker
        {
            let session = session.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(INTERNAL_TICK_INTERVAL);
                while !session.active.is_cancelled() {
                    match session.tick().await {
                        Ok(_) => (),
                        Err(e) => tracing::error!("{e}"),
                    }
                    interval.tick().await;
                }

                tracing::info!("Session ticker closed");
            });
        }

        // Packet processor
        {
            let session = session.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(TICK_INTERVAL);
                while !session.active.is_cancelled() {
                    match session.process_packet().await {
                        Ok(_) => (),
                        Err(e) => tracing::error!("{e}"),
                    }
                    interval.tick().await;
                }

                tracing::info!("Session processor closed");
            });
        }

        tracing::info!("Session {client_guid:X} created");
        session
    }

    async fn process_packet(&self) -> VexResult<()> {
        let task = tokio::select! {
            _ = self.active.cancelled() => {
                return Ok(())
            },
            task = self.queue.pop() => task
        };
        *self.last_update.write() = Instant::now();

        match *task.first().unwrap() {
            Ack::ID => self.handle_ack(task).await,
            Nack::ID => self.handle_nack(task).await,
            _ => self.handle_frame_set(task).await
        }
    }

    async fn handle_frame_set(&self, task: BytesMut) -> VexResult<()> {
        let frame_set = FrameSet::decode(task)?;
        for frame in frame_set.frames {
            if frame.is_compound {
                self.compound_collector.insert(frame);
            }
        }

        Ok(())
    }

    async fn handle_ack(&self, task: BytesMut) -> VexResult<()> {
        todo!("Handle ack");
    }

    async fn handle_nack(&self, task: BytesMut) -> VexResult<()> {
        todo!("Handle nack");
    }

    /// Performs tasks not related to packet processing
    async fn tick(self: &Arc<Self>) -> VexResult<()> {
        // Session has timed out
        if Instant::now().duration_since(*self.last_update.read()) > SESSION_TIMEOUT {
            self.active.cancel();
            tracing::info!("Session timed out");
        }

        Ok(())
    }

    #[inline]
    pub fn active(&self) -> bool {
        !self.active.is_cancelled()
    }

    fn forward(self: &Arc<Self>, buffer: BytesMut) {
        self.queue.push(buffer);
    }
}

pub struct SessionController {
    global_token: CancellationToken,
    map: DashMap<SocketAddr, Arc<Session>>,
    max_player_count: usize,
}

impl SessionController {
    pub fn new(
        global_token: CancellationToken,
        max_player_count: usize,
    ) -> VexResult<SessionController> {
        Ok(SessionController {
            global_token,
            map: DashMap::new(),
            max_player_count,
        })
    }

    pub fn add_session(&self, address: SocketAddr, client_guid: i64) {
        let session = Session::new(address, client_guid);
        self.map.insert(address, session);
    }

    pub fn forward_packet(&self, packet: RawPacket) -> VexResult<()> {
        self.map
            .get(&packet.address)
            .map(|r| {
                let session = r.value();
                session.forward(packet.buffer);
            })
            .ok_or(vex_error!(
                InvalidRequest,
                "Attempted to forward packet for non-existent session"
            ))
    }

    pub fn player_count(&self) -> usize {
        self.map.len()
    }

    pub fn max_player_count(&self) -> usize {
        self.max_player_count
    }
}
