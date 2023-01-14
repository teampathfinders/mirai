use std::sync::{Arc, RwLock};
use rand::Rng;
use crate::config::{CLIENT_VERSION_STRING, NETWORK_VERSION};
use crate::error::VexResult;
use crate::generate_getters;
use crate::raknet::{Session, SessionController};

pub struct ServerData {
    guid: i64,
    metadata: RwLock<String>

    // ipv4_port: u16
}

impl ServerData {
    pub fn new() -> VexResult<Self> {
        let mut data = Self {
            guid: rand::thread_rng().gen(),
            metadata: RwLock::new(String::new()),
            // ipv4_port
        };
        data.refresh_metadata("Standard description")?;
        tracing::debug!("SERVER GUID: {:#0X}", data.guid as u64);

        Ok(data)
    }

    pub fn guid(&self) -> i64 {
        self.guid
    }

    pub fn metadata(&self) -> VexResult<String> {
        let lock = self.metadata.read()?;
        Ok((*lock).clone())
    }

    fn refresh_metadata(&self, description: &str) -> VexResult<()> {
        let new_id = format!(
            "MCPE;Vex Dedicated Server;{};{};{};{};{};{};Survival;1;{};{};",
            NETWORK_VERSION,
            CLIENT_VERSION_STRING,
            // self.session_controller.player_count(),
            // self.session_controller.max_player_count(),
            0, 10,
            self.guid,
            description,
            // self.ipv4_port,
            19132,
            19133
        );

        let mut lock = self.metadata.write()?;
        *lock = new_id;

        Ok(())
    }
}