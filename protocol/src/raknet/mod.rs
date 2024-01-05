//! All types and packets implemented in the RakNet protocol.

use util::glob_export;

glob_export!(acknowledgements);
glob_export!(connection_request);
glob_export!(connection_request_accepted);
glob_export!(disconnect);
glob_export!(incompatible_protocol);
glob_export!(new_incoming_connection);
glob_export!(open_connection_reply1);
glob_export!(open_connection_reply2);
glob_export!(open_connection_request1);
glob_export!(open_connection_request2);
glob_export!(unconnected_ping);
glob_export!(unconnected_pong);
glob_export!(connected_ping);
glob_export!(connected_pong);

/// Version of Raknet that this server uses.
pub const RAKNET_VERSION: u8 = 11;
/// Special sequence of bytes that is contained in every unframed Raknet packet.
pub const OFFLINE_MESSAGE_DATA: &[u8] = &[
    0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78,
];
