
use crate::error::VexResult;
use dashmap::DashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

const TICK_INTERVAL: Duration = Duration::from_millis(1000 / 50);

#[derive(Debug)]
pub struct Session {
    address: SocketAddr,
    last_update: Instant,
    active: CancellationToken,
}

impl Session {
    pub fn new(address: SocketAddr) -> Arc<Self> {
        let session = Arc::new(Self {
            address,
            last_update: Instant::now(),
            active: CancellationToken::new(),
        });

        {
            let session = session.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(TICK_INTERVAL);
                while !session.active.is_cancelled() {
                    match session.tick().await {
                        Ok(_) => (),
                        Err(e) => tracing::error!("{e}"),
                    }
                    interval.tick().await;
                }
            });
        }

        session
    }

    #[inline]
    pub fn active(&self) -> bool {
        !self.active.is_cancelled()
    }

    async fn tick(self: Arc<Self>) -> VexResult<()> {
        Ok(())
    }
}

pub struct SessionController {
    global_token: CancellationToken,
    map: DashMap<SocketAddr, Session>,
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

    pub async fn start(&self) -> VexResult<()> {
        tracing::info!("Session service online");
        Ok(())
    }

    pub fn player_count(&self) -> usize {
        self.map.len()
    }

    pub fn max_player_count(&self) -> usize {
        self.max_player_count
    }
}
