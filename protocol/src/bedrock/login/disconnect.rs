use util::{BinaryWrite, VarString};
use util::Result;
use util::Serialize;

use crate::bedrock::ConnectedPacket;

pub const DISCONNECTED_NOT_AUTHENTICATED: &str =
    "disconnectionScreen.notAuthenticated";
pub const DISCONNECTED_NO_REASON: &str = "disconnectionScreen.noReason";
pub const DISCONNECTED_TIMEOUT: &str = "disconnectionScreen.timeout";
pub const DISCONNECTED_LOGIN_FAILED: &str = "disconnect.loginFailed";
pub const DISCONNECTED_ENCRYPTION_FAIL: &str =
    "Encryption checksums do not match.";
pub const DISCONNECTED_BAD_PACKET: &str = "Client sent bad packet.";

/// Reason why the client was disconnected.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DisconnectReason {
    Unknown,
    NoInternet,
    NoPermissions,
    UnrecoverableError,
    ThirdPartyBlocked,
    ThirdPartyNoInternet,
    ThirdPartyBadIp,
    ThirdPartyNoServerOrServerLocked,
    VersionMismatch,
    SkinIssue,
    InviteSessionNotFound,
    EduLevelSettingsMissing,
    LocalServerNotFound,
    LegacyDisconnect,
    UserLeaveGameAttempted,
    PlatformLockedSkinsError,
    RealmsWorldUnassigned,
    RealmsServerHidden,
    RealmsServerDisabledBeta,
    RealmsServerDisabled,
    CrossPlatformDisallowed,
    CannotConnect,
    SessionNotFound,
    ClientSettingsIncompatible,
    ServerFull,
    InvalidPlatformSkin,
    EditionVersionMismatch,
    EditionMismatch,
    LevelNewerThanExeVersion,
    NoFailOccurred,
    BannedSkin,
    Timeout,
    ServerNotFound,
    OutdatedServer,
    OutdatedClient,
    NoPremiumPlatform,
    MultiplayerDisabled,
    NoWifi,
    WorldCorruption,
    NoReason,
    Disconnected,
    InvalidPlayer,
    LoggedInOtherLocation,
    ServerIdConflict,
    NotAllowed,
    NotAuthenticated,
    InvalidTenant,
    UnknownPacket,
    UnexpectedPacket,
    InvalidCommandRequestPacket,
    HostSuspended,
    LoginPacketNoRequest,
    LoginPacketNoCert,
    MissingClient,
    Kicked,
    KickedForExploit,
    KickedForIdle,
    ResourcePackProblem,
    IncompatiblePack,
    OutOfStorage,
    InvalidLevel,
    DisconnectPacketDeprecated,
    BlockMismatch,
    InvalidHeights,
    InvalidWidths,
    ConnectionLost,
    ZombieConnection,
    Shutdown,
    ReasonNotSet,
    LoadingStateTimeout,
    ResourcePackLoadingFailed,
    SearchingForSessionLoadingScreenFailed,
    ConnProtocolVersion,
    SubsystemStatusError,
    EmptyAuthFromDiscovery,
    EmptyUrlFromDiscovery,
    ExpiredAuthFromDiscovery,
    UnknownSignalServiceSignInFailure,
    XblJoinLobbyFailure,
    UnspecifiedClientInstanceDisconnection,
    ConnSessionNotFound,
    ConnCreatePeerConnection,
    ConnIce,
    ConnConnectRequest,
    ConnNegotiationTimeout,
    ConnInactivityTimeout,
    StaleConnectionBeingReplaced,
    RealmsSessionNotFound,
    BadPacket
}

/// Sent by the server to disconnect a client.
#[derive(Debug, Clone)]
pub struct Disconnect<'a> {
    pub reason: DisconnectReason,
    /// Whether to immediately send the client to the main menu.
    pub hide_message: bool,
    /// Message to display to the client
    pub message: &'a str,
}

impl ConnectedPacket for Disconnect<'_> {
    const ID: u32 = 0x05;

    fn serialized_size(&self) -> usize {
        1 + self.message.var_len()
    }
}

impl Serialize for Disconnect<'_> {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        if self.message.is_empty() {
            /// An empty message will cause Minecraft to just ignore the disconnect packet and will
            /// cause all kinds of problems.
            anyhow::bail!("Disconnect message cannot be empty");
        }

        writer.write_var_i32(self.reason as i32); // Reason unknown
        writer.write_bool(self.hide_message)?;
        writer.write_str(self.message)
    }
}
