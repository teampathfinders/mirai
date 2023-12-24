use serde_repr::Deserialize_repr;

use util::bytes::{BinaryRead, SharedBuffer};
use util::Deserialize;
use util::Result;

use crate::network::ConnectedPacket;
use crate::crypto::{
    IdentityData, parse_identity_data, parse_user_data, UserData,
};
use crate::network::Skin;

/// Device operating system
#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize_repr)]
#[repr(u8)]
pub enum DeviceOS {
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
    Pocket,
}

/// Packet received by the client before initiating encryption.
/// A [`ServerToClientHandshake`](crate::network::ServerToClientHandshake) should be sent in response.
#[derive(Debug)]
pub struct Login {
    /// Identity data (Xbox account ID, username, etc.)
    pub identity: IdentityData,
    /// User data (device OS, language, etc.)
    pub user_data: UserData,
    /// Skin.
    pub skin: Skin,
}

impl ConnectedPacket for Login {
    const ID: u32 = 0x01;
}

impl Deserialize<'_> for Login {
    fn deserialize(mut buffer: SharedBuffer) -> anyhow::Result<Self> {
        let _version = buffer.read_u32_be()?; // Skip protocol version, use the one in RequestNetworkSettings instead.
        buffer.read_var_u32()?;

        let identity_data = parse_identity_data(&mut buffer)?;
        let data =
            parse_user_data(&mut buffer, &identity_data.public_key)?;

        Ok(Self {
            identity: IdentityData {
                uuid: identity_data.client_data.uuid,
                xuid: identity_data.client_data.xuid.parse()?,
                display_name: identity_data.client_data.display_name,
                public_key: identity_data.public_key,
            },
            user_data: data.data,
            skin: data.skin,
        })
    }
}
