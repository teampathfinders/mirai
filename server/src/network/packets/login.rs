use base64::Engine;
use bytes::{Buf, BytesMut};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use serde_repr::Deserialize_repr;
use spki::SubjectPublicKeyInfo;

use crate::crypto::{
    parse_identity_data, parse_user_data, IdentityData, UserData,
};
use crate::network::packets::GamePacket;
use crate::skin::Skin;
use common::Decodable;
use common::ReadExtensions;
use common::{bail, vassert};
use common::{VError, VResult};

/// Device operating system
#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize_repr)]
#[repr(u8)]
pub enum BuildPlatform {
    Android,
    Ios,
    Osx,
    FireOS,
    /// Samsung's GearVR
    GearVR,
    HoloLens,
    /// Windows 10/11 UWP variant of the game
    Win10,
    Win32,
    Dedicated,
    TvOS,
    /// Sometimes called Orbis.
    PlayStation,
    Nx,
    Xbox,
    WindowsPhone,
    Linux,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize_repr)]
#[repr(i32)]
pub enum UiProfile {
    Classic,
    Pocket
}

/// Packet received by the client before initiating encryption.
/// A [`ServerToClientHandshake`](super::ServerToClientHandshake) should be sent in response.
#[derive(Debug, Clone)]
pub struct Login {
    /// Identity data (Xbox account ID, username, etc.)
    pub identity: IdentityData,
    /// User data (device OS, language, etc.)
    pub user_data: UserData,
    /// Skin.
    pub skin: Skin
}

impl GamePacket for Login {
    const ID: u32 = 0x01;
}

impl Decodable for Login {
    fn decode(mut buffer: BytesMut) -> VResult<Self> {
        buffer.advance(4); // Skip protocol version, use the one in RequestNetworkSettings instead.
        buffer.get_var_u32()?;

        let identity_data = parse_identity_data(&mut buffer)?;
        let data =
            parse_user_data(&mut buffer, &identity_data.public_key)?;

        Ok(Self {
            identity: IdentityData {
                identity: identity_data.client_data.uuid,
                xuid: identity_data.client_data.xuid.parse()?,
                display_name: identity_data.client_data.display_name,
                public_key: identity_data.public_key,
            },
            user_data: data.data,
            skin: data.skin
        })
    }
}
