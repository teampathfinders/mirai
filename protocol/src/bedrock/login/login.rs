use serde_repr::Deserialize_repr;

use util::{BinaryRead, SharedBuffer};
use util::Deserialize;
use util::Result;

use crate::bedrock::ConnectedPacket;
use crate::crypto::{
    BedrockIdentity, parse_identity_data, parse_user_data, BedrockClientInfo,
};
use crate::bedrock::Skin;

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

/// The UI profile setting that the client is using.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize_repr)]
#[repr(i32)]
pub enum UiProfile {
    /// The classic UI profile.
    Classic,
    /// The Pocket Edition UI profile.
    Pocket,
}

/// Packet received by the client before initiating encryption.
/// A [`ServerToClientHandshake`](crate::bedrock::ServerToClientHandshake) should be sent in response.
#[derive(Debug)]
pub struct Login {
    /// Identity data (Xbox account ID, username, etc.)
    pub identity: BedrockIdentity,
    /// User data (device OS, language, etc.)
    pub client_info: BedrockClientInfo,
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
            identity: BedrockIdentity {
                uuid: identity_data.client_data.uuid,
                xuid: identity_data.client_data.xuid.parse()?,
                name: identity_data.client_data.display_name,
                public_key: identity_data.public_key,
            },
            client_info: data.data,
            skin: data.skin,
        })
    }
}
