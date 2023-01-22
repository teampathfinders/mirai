use base64::Engine;
use bytes::{Buf, BytesMut};
use serde_derive::Deserialize;
use crate::error::VexResult;
use crate::packets::GamePacket;
use crate::raknet::packets::Decodable;
use crate::util::ReadExtensions;
use crate::{vex_assert, vex_error};

#[derive(serde::Deserialize, Debug)]
pub struct JsonClientData {
    #[serde(rename = "XUID")]
    pub xuid: String,

    #[serde(rename = "displayName")]
    pub display_name: String,

    #[serde(rename = "identity")]
    pub uuid: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct TokenClaims {
    #[serde(rename = "extraData")]
    pub client_data: JsonClientData,
}

#[derive(serde::Deserialize, Debug)]
pub struct TokenChain {
    pub chain: Vec<String>,
}

#[derive(Debug)]
pub struct Login {
    pub protocol_version: u32,
    pub xuid: u64,
    pub uuid: u128,
    pub name: String,
}

impl GamePacket for Login {
    const ID: u32 = 0x01;
}

impl Decodable for Login {
    fn decode(mut buffer: BytesMut) -> VexResult<Self> {
        let protocol_version = buffer.get_u32();
        tracing::info!("protocol version {protocol_version}");

        buffer.get_var_u32()?;

        let client_data = decode_identity_data(&mut buffer)?;
        tracing::info!("{client_data:?}");
        todo!()
    }
}

fn decode_identity_data(buffer: &mut BytesMut) -> VexResult<TokenClaims> {
    let token_length = buffer.get_u32_le();
    tracing::info!("token length {token_length}");

    let position = buffer.len() - buffer.remaining();
    let token = &buffer.as_ref()[position..(position + token_length as usize)];

    let chains = match serde_json::from_slice::<TokenChain>(token) {
        Ok(c) => c,
        Err(e) => {
            return Err(vex_error!(InvalidRequest, format!("Invalid JSON: {e}")));
        }
    };

    let base64_engine = base64::engine::general_purpose::STANDARD_NO_PAD;
    for (index, chain) in chains.chain.iter().enumerate() {
        let mut split = chain.split('.');
        if let Some(second) = split.nth(1) {
            if let Ok(decoded) = base64_engine.decode(second) {
                if let Ok(json) = serde_json::from_slice(&decoded) {
                    return Ok(json);
                }
            }
        }
    }

    Err(vex_error!(InvalidRequest, "No identity data was found in the login tokens"))
}