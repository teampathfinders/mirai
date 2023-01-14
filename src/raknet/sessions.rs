use crate::error::VexResult;
use dashmap::DashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use crate::data::ServerData;

pub struct Session {
    address: SocketAddr,
}

pub struct SessionController {
    global_token: CancellationToken,
    map: DashMap<SocketAddr, Session>,
    max_player_count: usize,
}

impl SessionController {
    pub fn new(
        data: Arc<ServerData>,
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
