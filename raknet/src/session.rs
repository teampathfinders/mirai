use std::{net::SocketAddr, sync::{Arc, atomic::{AtomicBool, AtomicU16, AtomicU32, AtomicU64}}, time::Instant, mem::MaybeUninit};

use parking_lot::{Mutex, RwLock};
use tokio::{net::UdpSocket, sync::{broadcast, mpsc}};
use tokio_util::sync::CancellationToken;
use util::MutableBuffer;

use crate::{Compounds, SendQueues, Recovery, BroadcastPacket, OrderChannel};

const ORDER_CHANNEL_COUNT: usize = 5;
const OUTPUT_CHANNEL_SIZE: usize = 5;

pub struct UserCreateInfo {
    pub address: SocketAddr,
    pub mtu: u16,
    pub guid: u64,
    pub socket: Arc<UdpSocket>
}

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

impl RaknetUser {
    pub fn handle_disconnect(&self) {
        self.active.cancel();
    }

    pub fn new(info: UserCreateInfo, rx: mpsc::Receiver<MutableBuffer>) -> (Arc<Self>, mpsc::Receiver<MutableBuffer>) {
        let mut order_channels: [MaybeUninit<OrderChannel>; ORDER_CHANNEL_COUNT] = unsafe {
            MaybeUninit::uninit().assume_init()
        };
        
        for channel in &mut order_channels {
            channel.write(OrderChannel::new());
        }

        let order_channels = unsafe { 
            std::mem::transmute::<
                [MaybeUninit<OrderChannel>; ORDER_CHANNEL_COUNT], 
                [OrderChannel; ORDER_CHANNEL_COUNT]
            >(order_channels)
        };

        let (tx, rx) = mpsc::channel(OUTPUT_CHANNEL_SIZE);

        let state = Arc::new(RaknetUser {
            active: CancellationToken::new(),
            address: info.address,
            last_update: RwLock::new(Instant::now()),
            socket: info.socket,
            broadcast: todo!(),
            tick: AtomicU64::new(0),
            batch_number: AtomicU32::new(0),
            send: SendQueues::new(),
            acknowledged: Mutex::new(Vec::with_capacity(5)),
            recovery: Recovery::new(),
            mtu: info.mtu,
            acknowledge_index: AtomicU32::new(0),
            compound_index: AtomicU16::new(0),
            compounds: Compounds::new(),
            sequence_index: AtomicU32::new(0),
            order: order_channels,
            output: tx,
        });

        state.clone().start_packet_job(rx);
        state.clone().start_tick_job();

        (state, rx)
    }
}