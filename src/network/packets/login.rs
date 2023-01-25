use base64::Engine;
use bytes::{Buf, BytesMut};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use spki::SubjectPublicKeyInfo;

use crate::crypto::login::{parse_identity_data, parse_user_data, IdentityData, UserData};
use crate::error::{VexError, VexResult};
use crate::network::packets::GamePacket;
use crate::network::traits::Decodable;
use crate::util::ReadExtensions;
use crate::{bail, error, vex_assert};

#[derive(Debug)]
pub enum DeviceOS {
    Android,
    Ios,
    Osx,
    FireOS,
    GearVR,
    HoloLens,
    Win10,
    Win32,
    Dedicated,
    TvOS,
    PlayStation,
    Nx,
    Xbox,
    WindowsPhone,
    Linux,
}

impl TryFrom<u8> for DeviceOS {
    type Error = VexError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use DeviceOS::*;

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
            _ => bail!(InvalidRequest, "Invalid device OS"),
        })
    }
}

#[derive(Debug)]
pub struct Login {
    pub protocol_version: u32,
    pub identity: IdentityData,
    pub user_data: UserData,
}

impl GamePacket for Login {
    const ID: u32 = 0x01;
}

impl Decodable for Login {
    fn decode(mut buffer: BytesMut) -> VexResult<Self> {
        let protocol_version = buffer.get_u32();
        buffer.get_var_u32()?;

        let identity_data = parse_identity_data(&mut buffer)?;
        let user_data = parse_user_data(&mut buffer, &identity_data.public_key)?;

        Ok(Self {
            protocol_version,
            identity: IdentityData {
                identity: identity_data.client_data.uuid,
                xuid: identity_data.client_data.xuid.parse()?,
                title_id: identity_data.client_data.title_id.parse()?,
                display_name: identity_data.client_data.display_name,
                public_key: identity_data.public_key,
            },
            user_data: UserData {
                device_os: DeviceOS::try_from(user_data.device_os)?,
                language_code: user_data.language_code,
            },
        })
    }
}
