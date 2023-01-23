use bytes::{BufMut, BytesMut};

use crate::error::VexResult;
use crate::packets::GamePacket;
use crate::raknet::packets::Encodable;
use crate::util::WriteExtensions;

/// Behavior pack information.
#[derive(Debug)]
pub struct BehaviorPack {
    /// UUID of the behavior pack.
    /// Each behavior pack must have a unique UUID.
    pub uuid: String,
    /// Version of the behavior pack.
    /// This allows the client to cache behavior packs.
    pub version: String,
    /// Size of the compressed archive of the behavior pack in bytes.
    pub size: u64,
    /// Key used to decrypt the packet if it is encrypted.
    /// This is generally used for marketplace packs.
    pub content_key: String,
    /// Subpack name.
    pub subpack_name: String,
    /// Another UUID required for marketplace and encrypted behavior packs.
    pub content_identity: String,
    /// Whether the pack contains script.
    /// If it does, the pack will only be downloaded if the client supports scripting.
    pub has_scripts: bool,
}

/// Resource pack information
#[derive(Debug)]
pub struct ResourcePack {
    /// UUID of the resource pack.
    /// Each resource pack must have a unique UUID.
    pub uuid: String,
    /// Version of the resource pack.
    /// This allows the client to cache resource packs.
    pub version: String,
    /// Size of the compressed archive of the resource pack in bytes.
    pub size: u64,
    /// Key used to decrypt the pack if it is encrypted.
    /// This is generally used for marketplace packs.
    pub content_key: String,
    /// Subpack name.
    pub subpack_name: String,
    /// Another UUID required for marketplace and encrypted resource packs.
    pub content_identity: String,
    /// Whether the pack contains scripts.
    /// If it does, the pack will only be downloaded if the client supports scripting.
    pub has_scripts: bool,
    /// Whether the pack uses raytracing.
    pub rtx_enabled: bool,
}

/// Contains information about the addons used by the server.
/// This should be sent after sending the [`PlayStatus`](super::PlayStatus) packet with a
/// [`LoginSuccess`](super::Status::LoginSuccess) status.
#[derive(Debug)]
pub struct ResourcePacksInfo {
    /// Forces the client to accept the packs to be able to join the server.
    pub required: bool,
    /// Indicates whether there are packs that make use of scripting.
    pub scripting_enabled: bool,
    /// List of behavior packs
    pub behavior_info: Vec<ResourcePack>,
    /// List of resource packs.
    pub resource_info: Vec<BehaviorPack>,
    /// Unknown what this does.
    pub forcing_server_packs: bool,
}

impl GamePacket for ResourcePacksInfo {
    const ID: u32 = 0x06;
}

impl Encodable for ResourcePacksInfo {
    fn encode(&self) -> VexResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_u8(self.required as u8);


        Ok(buffer)
    }
}