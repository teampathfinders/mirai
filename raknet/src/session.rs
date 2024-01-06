use std::{net::SocketAddr, sync::{Arc, atomic::{AtomicBool, AtomicU16, AtomicU32}}, time::Instant};

use parking_lot::{Mutex, RwLock};
use tokio::net::UdpSocket;

use crate::raknet::{Compounds, OrderChannel, Recovery, SendQueues};

const ORDER_CHANNEL_COUNT: usize = 5;

/// The Raknet layer of the user. This handles the entire Raknet protocol for the client.
pub struct RaknetUser {
    // Networking
    pub address: SocketAddr,
    pub socket: Arc<UdpSocket>,

    // Inter-session communication
    pub broadcast: broadcast::Sender<BroadcastPacket>,

    // Raknet data
    pub active: CancellationToken,
    pub mtu: u16,
    /// Keeps track of when the last update was received from the client.
    /// This enables disconnecting users that have lost connection to the server.
    pub last_update: RwLock<Instant>,
    pub tick: AtomicU64,
    /// This client's current batch number. It is increased for every packet batch sent.
    pub batch_number: AtomicU32,

    pub send: SendQueues,

    /// Wrapped in a mutex since reading this will also clear it.
    pub acknowledged: Mutex<Vec<u32>>,
    pub acknowledge_index: AtomicU32,

    pub compound_index: AtomicU16,
    pub compounds: Compounds,

    pub recovery: Recovery,

    pub sequence_index: AtomicU32,
    pub order: [OrderChannel; ORDER_CHANNEL_COUNT],

    pub output: mpsc::Sender<MutableBuffer>
}