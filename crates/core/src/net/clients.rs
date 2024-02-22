use std::net::SocketAddr;
use std::sync::{Arc, OnceLock, Weak};
use std::time::Duration;

use anyhow::Context;
use dashmap::DashMap;

use proto::uuid::Uuid;
use raknet::{BroadcastPacket, RakNetCreateDescription, RakNetClient};
use proto::bedrock::{ConnectedPacket, Disconnect, DisconnectReason};
use util::{RVec, Joinable, Serialize};

use tokio::sync::{broadcast, mpsc};
use tokio::task::{JoinHandle, JoinSet};
use tokio_util::sync::CancellationToken;

use crate::instance::Instance;

use super::{ForwardablePacket, BedrockClient};

const BROADCAST_CHANNEL_CAPACITY: usize = 5;
const FORWARD_TIMEOUT: Duration = Duration::from_millis(10);

/// Contains the user state itself and a method to contact the user.
pub struct UserMapEntry<T> {
    channel: mpsc::Sender<RVec>,
    state: Arc<T>
}

impl<T> UserMapEntry<T> {
    /// Forwards a packet to the user for processing.
    #[inline]
    #[allow(clippy::future_not_send)]
    pub async fn forward(&self, packet: RVec) -> anyhow::Result<()> {
        self.channel.send_timeout(packet, FORWARD_TIMEOUT).await.context("Server-side client timed out")?;
        Ok(())
    }
}

/// Keeps track of all users currently connected to the server.
pub struct Clients {
    /// Token that indicates whether this user map has fully shut down.
    /// 
    /// This is used by other services to wait for all players to disconnect before
    /// shutting down.
    shutdown_token: CancellationToken,
    
    connecting_map: Arc<DashMap<SocketAddr, UserMapEntry<RakNetClient>>>,
    connected_map: Arc<DashMap<SocketAddr, UserMapEntry<BedrockClient>>>,
    /// Channel that sends a packet to all connected sessions.
    broadcast: broadcast::Sender<BroadcastPacket>,

    commands: Arc<crate::command::Service>,
    level: Arc<crate::level::Service>,
    instance: OnceLock<Weak<Instance>>
}

impl Clients {
    /// Creates a new user map.
    pub fn new(commands: Arc<crate::command::Service>, level: Arc<crate::level::Service>) -> Self {
        let connecting_map = Arc::new(DashMap::new());
        let connected_map = Arc::new(DashMap::new());

        let (broadcast, _) = broadcast::channel(BROADCAST_CHANNEL_CAPACITY);

        Self {
            shutdown_token: CancellationToken::new(),
            connecting_map, 
            connected_map, 
            broadcast, 
            commands, 
            level,
            instance: OnceLock::new()
        }
    }   

    /// Inserts a user into the map.
    pub(crate) fn insert(&self, info: RakNetCreateDescription) {
        let (tx, rx) = mpsc::channel(BROADCAST_CHANNEL_CAPACITY);

        let address = info.address;
        let (state, state_rx) = 
            RakNetClient::new(info, self.broadcast.clone(), rx);
        
        let connecting_map = Arc::clone(&self.connecting_map);
        let connected_map = Arc::clone(&self.connected_map);
        let broadcast = self.broadcast.clone();
        let endpoint = Arc::clone(&self.commands);
        let level = Arc::clone(&self.level);

        // Instance should exist while the user map exists.
        #[allow(clippy::unwrap_used)]
        let instance = Weak::clone(self.instance.get().unwrap());

        // Callback to move the client from the connecting map to the connected map.
        // This is done when the Raknet layer attempts to send a message to the Bedrock layer
        // signalling that the Raknet connection is fully set up.
        tokio::spawn(async move {
            if let Some((_, raknet_user)) = connecting_map.remove(&address) {
                let bedrock_user = UserMapEntry {
                    channel: raknet_user.channel, state: BedrockClient::new(
                        raknet_user.state, 
                        state_rx, 
                        endpoint, 
                        level, 
                        broadcast,
                        instance
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

    /// Sets the instance pointer for this service.
    /// 
    /// This is used to access data from other services.
    pub(crate) fn set_instance(&self, instance: &Arc<Instance>) -> anyhow::Result<()> {
        self.instance.set(Arc::downgrade(instance)).map_err(|_| anyhow::anyhow!("Client service instance was already set"))
    }

    /// Returns the instance that owns this service.
    fn instance(&self) -> Arc<Instance> {
        // This will not panic because the instance field is initialised before the first command can be executed.
        #[allow(clippy::unwrap_used)]
        self.instance.get().unwrap().upgrade().unwrap()
    }

    /// Attempts to retrieve the user with the given XUID.
    pub fn by_xuid(&self, xuid: u64) -> Option<Arc<BedrockClient>> {
        todo!()
    }

    /// Attempts to retrieve the user with the given UUID.
    pub fn by_uuid(&self, uuid: Uuid) -> Option<Arc<BedrockClient>> {
        todo!()
    }

    /// Attempts to retrieve the user with the given IP address.
    pub fn by_address(&self, address: &SocketAddr) -> Option<Arc<BedrockClient>> {
        self.connected_map
            .get(address)
            .map(|r| Arc::clone(&r.value().state))
    }

    /// Attempts to retrieve the user with the given username.
    pub fn by_username<S: AsRef<str>>(&self, username: S) -> Option<Arc<BedrockClient>> {
        todo!()
    }

    /// Forwards a packet to a user within the map.
    pub(crate) async fn forward(&self, packet: ForwardablePacket) -> anyhow::Result<()> {
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
    #[inline]
    pub fn total_connecting(&self) -> usize {
        self.connecting_map.len()
    }

    /// How many users are fully connected to the server.
    #[inline]
    pub fn total_connected(&self) -> usize {
        self.connected_map.len()
    }

    /// Maximum amount of concurrently connected users.
    pub fn max_connections(&self) -> usize {
        self.instance().config().max_connections()
    }

    /// Signals the user map to shut down.
    /// 
    /// This function returns a handle that can be used to await shutdown.
    pub(crate) fn shutdown(self: &Arc<Clients>) -> JoinHandle<anyhow::Result<()>> {
        let this = Arc::clone(self);
        tokio::spawn(async move {
            tracing::info!("Disconnecting all clients");

            // Ignore result because it can only fail if there are no receivers remaining.
            // In that case this shouldn't do anything anyways.
            // this.broadcast(
            //     Disconnect {
            //         reason: DisconnectReason::Shutdown,
            //         hide_message: false,
            //         message: "disconnect.disconnected"
            //     }
            // )?;

            let mut join_set = JoinSet::new();
            this.connecting_map.retain(|_, user| {
                user.state.disconnect();

                let clone = Arc::clone(&user.state);
                join_set.spawn(async move { clone.join().await });

                false
            });

            this.connected_map.retain(|_, user| {
                let _: anyhow::Result<()> = user.state.send(Disconnect {
                    hide_message: false,
                    message: "Server shutting down",
                    reason: DisconnectReason::Shutdown
                });
                user.state.raknet.active.cancel();

                let clone = Arc::clone(&user.state);
                join_set.spawn(async move { clone.join().await });

                false
            });

            // Await all shutdown listeners.
            while join_set.join_next().await.is_some() {}

            this.shutdown_token.cancel();

            tracing::info!("All clients succesfully disconnected");

            Ok(())
        })
    }
}

impl Joinable for Clients {
    /// Sends a [`Disconnect`] packet to every connected user and waits for all users
    /// to acknowledge they have been disconnected.
    /// 
    /// In case a user does not respond, there is a 2 second timeout.
    async fn join(&self) -> anyhow::Result<()> {
        self.shutdown_token.cancelled().await;

        Ok(())
    }
}
