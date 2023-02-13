use base64::Engine;
use bytes::{Buf, BytesMut};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use spki::SubjectPublicKeyInfo;

use crate::crypto::{parse_identity_data, parse_user_data, IdentityData, UserData};
use crate::network::packets::GamePacket;
use common::Decodable;
use common::ReadExtensions;
use common::{bail, vassert};
use common::{VError, VResult};

/// Device operating system
#[derive(Debug, Copy, Clone)]
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

impl TryFrom<u8> for BuildPlatform {
    type Error = VError;

    fn try_from(value: u8) -> VResult<Self> {
        use BuildPlatform::*;

        Ok(match value {
            1 => Android,
            2 => Ios,
            3 => Osx,
            4 => FireOS,
            5 => GearVR,
            6 => HoloLens,
            7 => Win10,
            8 => Win32,
            9 => Dedicated,
            10 => TvOS,
            11 => PlayStation,
            12 => Nx,
            13 => Xbox,
            14 => WindowsPhone,
            15 => Linux,
            _ => bail!(BadPacket, "Invalid device OS {}, expected 1-15", value),
        })
    }
}

/// Packet received by the client before initiating encryption.
/// A [`ServerToClientHandshake`](super::ServerToClientHandshake) should be sent in response.
#[derive(Debug)]
pub struct Login {
    /// Identity data (Xbox account ID, username, etc.)
    pub identity: IdentityData,
    /// User data (device OS, skin, etc.)
    pub user_data: UserData,
}

impl GamePacket for Login {
    const ID: u32 = 0x01;
}

impl Decodable for Login {
    fn decode(mut buffer: BytesMut) -> VResult<Self> {
        buffer.advance(4); // Skip protocol version, use the one in RequestNetworkSettings instead.
        buffer.get_var_u32()?;

        let identity_data = parse_identity_data(&mut buffer)?;
        let user_data = parse_user_data(&mut buffer, &identity_data.public_key)?;

        Ok(Self {
            identity: IdentityData {
                identity: identity_data.client_data.uuid,
                xuid: identity_data.client_data.xuid.parse()?,
                title_id: identity_data.client_data.title_id.parse()?,
                display_name: identity_data.client_data.display_name,
                public_key: identity_data.public_key,
            },
            user_data: UserData {
                device_os: BuildPlatform::try_from(user_data.device_os)?,
                language_code: user_data.language_code,
            },
        })
    }
}
