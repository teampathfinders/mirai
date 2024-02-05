use std::{net::SocketAddr, sync::{Arc, atomic::{AtomicU16, AtomicU32, AtomicU64}}, time::Instant, mem::MaybeUninit};

use parking_lot::{Mutex, RwLock};
use proto::raknet::DisconnectNotification;
use tokio::{net::UdpSocket, sync::{broadcast, mpsc, Semaphore}};
use tokio_util::sync::CancellationToken;
use util::Joinable;

use crate::{BroadcastPacket, Compounds, OrderChannel, Recovery, Reliability, SendConfig, SendPriority, SendQueues, BUDGET_SIZE};

const ORDER_CHANNEL_COUNT: usize = 5;
const OUTPUT_CHANNEL_SIZE: usize = 5;
/// A command that the Raknet layer will send to its parent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RaknetCommand {
    /// The client has exhausted its budget and should be disconnected.
    /// An exhausted budget might be the result of a DOS attack.
    /// 
    /// This mechanism prevents flooding by rate limiting requests.
    BudgetExhausted,
    /// The Raknet client has disconnected.
    Disconnected,
    /// The Raknet layer has received a packet and finished preprocessing it.
    Received(Vec<u8>)
}

/// Information required to create a new RakNet user.
pub struct RaknetCreateInfo {
    /// IP address of the client.
    pub address: SocketAddr,
    /// Maximum transfer unit of the client.
    pub mtu: u16,
    /// RakNet guid of the client. This is provided by the client and is therefore not
    /// a secure way to identity clients.
    pub guid: u64,
    /// UDP socket that is connected to the client.
    pub socket: Arc<UdpSocket>
}

/// The Raknet layer of the user. This handles the entire Raknet protocol for the client.
pub struct RaknetUser {
    /// Keeps track of the remaining "budget" of this user.
    /// This is used to implement rate limiting.
    pub budget: Semaphore,
    /// IP address of the user.
    pub address: SocketAddr,
    /// Socket used for communication with this user.
    pub socket: Arc<UdpSocket>,
    /// Channel that can perform inter-user packet broadcasting.
    pub broadcast: broadcast::Sender<BroadcastPacket>,
    /// Whether the user is still active.
    /// Cancelling this token means that all pending packets will be flushed and the server will process no more
    /// packets coming from this user.
    pub active: CancellationToken,
    /// Maximum transfer unit. This is maximum size of a single packet. If a packet exceeds this size
    /// it will split into multiple fragments.
    pub mtu: u16,
    /// Keeps track of when the last update was received from the client.
    /// This enables disconnecting users that have lost connection to the server.
    pub last_update: RwLock<Instant>,
    /// Increased for every round of packets processed.
    pub tick: AtomicU64,
    /// This client's current batch number. It is increased for every packet batch sent.
    pub batch_number: AtomicU32,
    /// Packets pending submission to the client.
    pub send: SendQueues,
    /// Pending acknowledgements.
    /// Wrapped in a mutex since reading this will also clear it.
    pub acknowledged: Mutex<Vec<u32>>,
    /// Current acknowledgement index.
    /// This is increased for every reliable packet sent.
    pub acknowledge_index: AtomicU32,
    /// Current compound index. This index uniquely identifies a compound of fragments.
    pub compound_index: AtomicU16,
    /// Collection of incomplete compounds. These compounds will slowly be filled up and
    /// will be processed when all fragments have been received.
    pub compounds: Compounds,
    /// Stores packets for recovery in case of packet loss.
    pub recovery: Recovery,
    /// Current sequence index, this is increased for every sequenced packet sent.
    pub sequence_index: AtomicU32,
    /// Multiple channels that ensure packets are received in the right order.
    pub order: [OrderChannel; ORDER_CHANNEL_COUNT],
    /// Channel used to submit packets that have been fully processed by the RakNet layer.
    /// These packets go on to be processed further by protocols running on top of RakNet
    /// such as the Minecraft Bedrock protocol.
    pub output: mpsc::Sender<RaknetCommand>,
    /// Handle to the processing job of this RakNet client.
    pub job_handle: RwLock<Option<tokio::task::JoinHandle<()>>>
}

impl RaknetUser {
    /// Creates a new RakNet user with the specified info.
    pub fn new(
        info: RaknetCreateInfo, 
        broadcast: broadcast::Sender<BroadcastPacket>,
        forward_rx: mpsc::Receiver<Vec<u8>>
    ) -> (Arc<Self>, mpsc::Receiver<RaknetCommand>) {
        // SAFETY: MaybeUninit does not require initialization, so it is safe to create an array
        // of them like this.
        let mut order_channels: [MaybeUninit<OrderChannel>; ORDER_CHANNEL_COUNT] = unsafe {
            MaybeUninit::uninit().assume_init()
        };
        
        for channel in &mut order_channels {
            channel.write(OrderChannel::new());
        }

        // SAFETY: This is safe because `MaybeUninit<T>` has the same memory layout as `T`.
        // It is safe to transmute between them.
        let order_channels = unsafe { 
            std::mem::transmute::<
                [MaybeUninit<OrderChannel>; ORDER_CHANNEL_COUNT], 
                [OrderChannel; ORDER_CHANNEL_COUNT]
            >(order_channels)
        };

        let (output_tx, output_rx) = mpsc::channel(OUTPUT_CHANNEL_SIZE);

        let state = Arc::new(RaknetUser {
            budget: Semaphore::new(BUDGET_SIZE),
            active: CancellationToken::new(),
            address: info.address,
            last_update: RwLock::new(Instant::now()),
            socket: info.socket,
            broadcast,
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
            output: output_tx,
            job_handle: RwLock::new(None)
        });

        let handle = tokio::spawn(Arc::clone(&state).receiver(forward_rx));
        *state.job_handle.write() = Some(handle);
    
        (state, output_rx)
    }

    /// Sends a RakNet disconnect packet to the client.
    pub fn disconnect(&self) {
        self.send_raw_buffer_with_config(vec![DisconnectNotification::ID], SendConfig {
            reliability: Reliability::Reliable,
            priority: SendPriority::High
        });
    }
}

impl Joinable for RaknetUser {
    #[tracing::instrument(
        skip_all,
        name = "RaknetUser::join",
        fields(
            %address = %self.address
        )
    )]
    async fn join(&self) -> anyhow::Result<()> {
        let handle = self.job_handle.write().take();
        if let Some(handle) = handle {
            // self.disconnect();
            match self.flush_all().await {
                Ok(_) => (),
                Err(e) => tracing::error!("Failed to flush last packets before shutdown: {e:#?}")
            }

            self.active.cancel();
            match handle.await {
                Ok(_) => Ok(()),
                Err(err) => {
                    tracing::error!("Error occurred while awaiting RakNet user service shutdown: {err:#?}");
                    Ok(())
                }
            }
        } else {
            tracing::error!("This RakNet user service has already been joined");
            anyhow::bail!("RakNet user service already joined");
        }
    }
}