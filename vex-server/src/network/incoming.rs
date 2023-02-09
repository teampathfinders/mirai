use std::io::Read;
use std::sync::atomic::Ordering;

use bytes::{Buf, BytesMut};

use vex_common::{bail, CompressionAlgorithm, ReadExtensions, SERVER_CONFIG, vassert, VResult};

use crate::network::header::Header;
use crate::network::packets::{ClientCacheStatus, ClientToServerHandshake, GamePacket, Login, RequestNetworkSettings, ResourcePackClientResponse, ViolationWarning};
use crate::network::Player;

impl Player {
    async fn handle_game_packet(&self, mut packet: BytesMut) -> VResult<()> {
        vassert!(packet.get_u8() == 0xfe);

        // Decrypt packet
        if self.encryptor.initialized() {
            // Safe to unwrap because the encryptor is confirmed to exist.
            let encryptor = self
                .encryptor
                .get()
                .expect("Encryptor was destroyed while it was in use");

            packet = match encryptor.decrypt(packet) {
                Ok(t) => t,
                Err(e) => {
                    return Err(e);

                    // if matches!(e, VError::BadPacket(_)) {
                    //     todo!();
                    // }

                    // TODO
                    // todo!("Disconnect client because of invalid packet");
                }
            }
        }

        let compression_enabled = self.compression_enabled.load(Ordering::SeqCst);
        let compression_threshold = SERVER_CONFIG.read().compression_threshold;

        if compression_enabled
            && compression_threshold != 0
            && packet.len() > compression_threshold as usize
        {
            // Packet is compressed
            let decompressed = match SERVER_CONFIG.read().compression_algorithm {
                CompressionAlgorithm::Snappy => {
                    todo!("Snappy decompression");
                }
                CompressionAlgorithm::Deflate => {
                    let mut reader = flate2::read::DeflateDecoder::new(packet.as_ref());
                    let mut decompressed = Vec::new();
                    reader.read_to_end(&mut decompressed)?;
                    // .context("Failed to decompress packet using Deflate")?;

                    BytesMut::from(decompressed.as_slice())
                }
            };

            self.handle_decompressed_game_packet(decompressed).await
        } else {
            self.handle_decompressed_game_packet(packet).await
        }
    }

    async fn handle_decompressed_game_packet(&self, mut packet: BytesMut) -> VResult<()> {
        let length = packet.get_var_u32()?;
        let header = Header::decode(&mut packet)?;

        match header.id {
            RequestNetworkSettings::ID => self.handle_request_network_settings(packet),
            Login::ID => self.handle_login(packet).await,
            ClientToServerHandshake::ID => self.handle_client_to_server_handshake(packet),
            ClientCacheStatus::ID => self.handle_client_cache_status(packet),
            ResourcePackClientResponse::ID => self.handle_resource_pack_client_response(packet),
            ViolationWarning::ID => self.handle_violation_warning(packet),
            id => bail!(BadPacket, "Invalid game packet: {:#04x}", id),
        }
    }
}