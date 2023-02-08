pub use compound_collector::*;
pub use order_channel::*;
pub use receive::*;
pub use recovery_queue::*;
pub use send::*;
pub use send_queue::*;
pub use session::*;
pub use tracker::*;

mod compound_collector;
mod order_channel;
mod receive;
mod recovery_queue;
mod send;
mod send_queue;
mod session;
mod tracker;

pub mod handlers;

/// ID of Minecraft game packets.
pub const GAME_PACKET_ID: u8 = 0xfe;
/// Protocol version that this server supports.
pub const NETWORK_VERSION: u32 = 567;
pub const CLIENT_VERSION_STRING: &str = "1.19.60";