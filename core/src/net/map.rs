use std::net::SocketAddr;

use std::sync::{Arc};
use std::time::Duration;

use anyhow::Context;
use dashmap::DashMap;

use raknet::{BroadcastPacket, RaknetCreateInfo, RaknetUser};

use tokio::sync::{broadcast, mpsc};
use proto::bedrock::{ConnectedPacket, Disconnect, DisconnectReason};
use replicator::Replicator;


use tokio::task::JoinSet;
use util::{BVec, Joinable, Serialize};


use crate::config::SERVER_CONFIG;

use super::{ForwardablePacket, BedrockUser};

const BROADCAST_CHANNEL_CAPACITY: usize = 5;
const FORWARD_TIMEOUT: Duration = Duration::from_millis(10);

/// Contains the user state itself and a method to contact the user.
pub struct UserMapEntry<T> {
    channel: mpsc::Sender<BVec>,
    state: Arc<T>
}

impl<T> UserMapEntry<T> {
    /// Forwards a packet to the user for processing.
    #[inline]
    #[allow(clippy::future_not_send)]
    pub async fn forward(&self, packet: BVec) -> anyhow::Result<()> {
        self.channel.send_timeout(packet, FORWARD_TIMEOUT).await.context("Server-side client timed out")?;
        Ok(())
    }
}

/// Keeps track of all users currently connected to the server.
pub struct UserMap {
    replicator: Arc<Replicator>,
    
    connecting_map: Arc<DashMap<SocketAddr, UserMapEntry<RaknetUser>>>,
    connected_map: Arc<DashMap<SocketAddr, UserMapEntry<BedrockUser>>>,
    /// Channel that sends a packet to all connected sessions.
    broadcast: broadcast::Sender<BroadcastPacket>,

    commands: Arc<crate::command::Service>,
    level: Arc<crate::level::Service>
}

impl UserMap {
    /// Creates a new user map.
    pub fn new(replicator: Arc<Replicator>, commands: Arc<crate::command::Service>, level: Arc<crate::level::Service>) -> Self {
        let connecting_map = Arc::new(DashMap::new());
        let connected_map = Arc::new(DashMap::new());

        let (broadcast, _) = broadcast::channel(BROADCAST_CHANNEL_CAPACITY);

        Self {
            replicator, connecting_map, connected_map, broadcast, commands, level
        }
    }   

    /// Sets the level instance that the user map should use.
    // pub fn set_level(&self, level: Arc<Level>) {
    //     if self.level.set(level).is_err() {
    //         tracing::error!("Level reference was already set");
    //     }
    // }

    /// Inserts a user into the map.
    pub fn insert(&self, info: RaknetCreateInfo) {
        let (tx, rx) = mpsc::channel(BROADCAST_CHANNEL_CAPACITY);

        let address = info.address;
        let (state, state_rx) = 
            RaknetUser::new(info, self.broadcast.clone(), rx);
        
        let connecting_map = Arc::clone(&self.connecting_map);
        let connected_map = Arc::clone(&self.connected_map);
        let replicator = Arc::clone(&self.replicator);
        let broadcast = self.broadcast.clone();
        let endpoint = Arc::clone(&self.commands);
        let level = Arc::clone(&self.level);

        // Callback to move the client from the connecting map to the connected map.
        // This is done when the Raknet layer attempts to send a message to the Bedrock layer
        // signalling that the Raknet connection is fully set up.
        tokio::spawn(async move {
            if let Some((_, raknet_user)) = connecting_map.remove(&address) {
                let bedrock_user = UserMapEntry {
                    channel: raknet_user.channel, state: BedrockUser::new(
                        raknet_user.state, replicator, state_rx, endpoint, level, broadcast
                    )
                };

                connected_map.insert(address, bedrock_user);
            } else {
                tracing::error!("Raknet client exists but is not tracked by user map");
            }
        });

        let connecting_map = Arc::clone(&self.connecting_map);
        let connected_map = Arc::clone(&self.connected_map);
        let state_clone = Arc::clone(&state);

        tokio::spawn(async move {
            state_clone.active.cancelled().await;
            connected_map.remove(&state_clone.address);
            connecting_map.remove(&state_clone.address);
        });

        self.connecting_map.insert(address, UserMapEntry {
            channel: tx, state
        });
    }

    /// Forwards a packet to a user within the map.
    pub async fn forward(&self, packet: ForwardablePacket) -> anyhow::Result<()> {
        if let Some(user) = self.connected_map.get(&packet.addr) {
            return user.channel.send_timeout(packet.buf, FORWARD_TIMEOUT)
                .await
                .context("Forwarding packet to user timed out")
        }

        if let Some(user) = self.connecting_map.get(&packet.addr) {
            return user.channel.send_timeout(packet.buf, FORWARD_TIMEOUT)
                .await
                .context("Forwarding packet to connecting user timed out")
        }

        Ok(())
    }

    /// Broadcasts the given packet to every client connected to the server.
    pub fn broadcast<T: ConnectedPacket + Serialize>(&self, packet: T) -> anyhow::Result<()> {
        // Broadcasting while there are no receivers will cause an error.
        if self.broadcast.receiver_count() != 0 {
            self.broadcast.send(BroadcastPacket::new(packet, None)?)?;
        }

        Ok(())
    }

    /// How many clients are currently in the process of logging in.
    pub fn connecting_count(&self) -> usize {
        self.connecting_map.len()
    }

    /// How many users are fully connected to the server.
    pub fn connected_count(&self) -> usize {
        self.connected_map.len()
    }

    /// Maximum amount of concurrently connected users.
    pub fn max_count(&self) -> usize {
        SERVER_CONFIG.read().max_players
    }
}

impl Joinable for UserMap {
    /// Sends a [`Disconnect`] packet to every connected user and waits for all users
    /// to acknowledge they have been disconnected.
    /// 
    /// In case a user does not respond, there is a 2 second timeout.
    ///
    /// # Errors
    /// 
    /// This function returns an error when the [`Disconnect`] packet fails to serialize.
    async fn join(&self) -> anyhow::Result<()> {
        tracing::info!("Disconnecting all clients");

        // Ignore result because it can only fail if there are no receivers remaining.
        // In that case this shouldn't do anything anyways.
        self.broadcast(
            Disconnect {
                reason: DisconnectReason::Shutdown,
                hide_message: false,
                message: "disconnect.disconnected"
            }
        )?;

        let mut join_set = JoinSet::new();
        self.connecting_map.retain(|_, user| {
            user.state.disconnect();

            let clone = Arc::clone(&user.state);
            join_set.spawn(async move { clone.join().await });

            false
        });

        self.connected_map.retain(|_, user| {
            let clone = Arc::clone(&user.state);
            join_set.spawn(async move { clone.join().await });

            false
        });

        // Await all shutdown listeners.
        while join_set.join_next().await.is_some() {}

        tracing::info!("All clients succesfully disconnected");

        Ok(())
    }
}
