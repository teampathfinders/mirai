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
    client_guid: i64,

    last_update: Instant,
    active: CancellationToken,
}

impl Session {
    pub fn new(address: SocketAddr, client_guid: i64) -> Arc<Self> {
        let session = Arc::new(Self {
            address,
            client_guid,
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

        tracing::info!("Session {client_guid:X} created");
        session
    }

    #[inline]
    pub fn active(&self) -> bool {
        !self.active.is_cancelled()
    }

    async fn tick(self: &Arc<Self>) -> VexResult<()> {
        Ok(())
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

    pub fn player_count(&self) -> usize {
        self.map.len()
    }

    pub fn max_player_count(&self) -> usize {
        self.max_player_count
    }
}
