use base64::Engine;
use bytes::{Buf, BytesMut};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde_derive::Deserialize;
use spki::SubjectPublicKeyInfo;

use crate::{bail, error, vex_assert};
use crate::error::VexResult;
use crate::network::packets::GamePacket;
use crate::network::traits::Decodable;
use crate::util::ReadExtensions;

pub const MOJANG_PUBLIC_KEY: &str = "MHYwEAYHKoZIzj0CAQYFK4EEACIDYgAE8ELkixyLcwlZryUQcu1TvPOmI2B7vX83ndnWRUaXm74wFfa5f/lwQNTfrLVHa2PmenpGI6JhIMUJaWZrjmMj90NoKNFSNBuKdm8rYiXsfaz3K36x/1U26HpG0ZxK/V1V";
const ENGINE: base64::engine::GeneralPurpose = base64::engine::general_purpose::STANDARD_NO_PAD;

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
    pub uuid: String,
    pub display_name: String,
    pub public_key: String
}

impl GamePacket for Login {
    const ID: u32 = 0x01;
}

impl Decodable for Login {
    fn decode(mut buffer: BytesMut) -> VexResult<Self> {
        let protocol_version = buffer.get_u32();
        buffer.get_var_u32()?;

        parse_identity_data(&mut buffer).unwrap();


        todo!();

        // let claims_data = decode_identity_data(&mut buffer)?;
        //
        // Ok(Self {
        //     protocol_version,
        //     xuid: claims_data.client_data.xuid.parse().unwrap(),
        //     uuid: claims_data.client_data.uuid,
        //     display_name: claims_data.client_data.display_name,
        // })
    }
}

//
// fn parse_claim(claim: &str, key: &[u8]) -> VexResult<()> {
//     let decoding_key = DecodingKey::from_ec_der(key);
//     let validation = Validation::new(Algorithm::ES384);
//     let header = jsonwebtoken::decode::<TokenClaims>(claim, &decoding_key, &validation)?;
//     tracing::debug!("{header:?}");
//
//     Ok(())
// }

fn parse_identity_data(buffer: &mut BytesMut) -> VexResult<()> {
    let token_length = buffer.get_u32_le();
    let position = buffer.len() - buffer.remaining();
    let token = &buffer.as_ref()[position..(position + token_length as usize)];

    let tokens = serde_json::from_slice::<TokenChain>(token)?;
    if tokens.chain.is_empty() {
        return Err(error!(InvalidRequest, format!("Client sent {} tokens, expected 3", tokens.chain.len())));
    }
    tracing::info!("{tokens:?}");

    let client_token = jsonwebtoken::decode_header(&tokens.chain[0])?;
    let raw_client_key = client_token.x5u.ok_or(error!(InvalidRequest, "Missing x5u in client-signed token"))?;
    let decoded_raw = ENGINE.decode(raw_client_key).unwrap();
    let info = spki::SubjectPublicKeyInfo::try_from(decoded_raw.as_ref()).unwrap();
    tracing::debug!("{info:?}");

    let decoding_key = DecodingKey::from_ec_der(info.subject_public_key);
    let validation = Validation::new(Algorithm::ES384);
    let client_token = jsonwebtoken::decode::<()>(&tokens.chain[0], &decoding_key, &validation).unwrap();
    tracing::info!("{client_token:?}");

    buffer.advance(token_length as usize);

    // Skin data
    // =========================

    let raw_token_length = buffer.get_u32_le();
    tracing::info!("Raw length {raw_token_length}");

    // let x509 = openssl::ec::EcKey::public_key_from_der(&ENGINE.decode(MOJANG_PUBLIC_KEY).unwrap()).unwrap();
    //
    // let client_token = &tokens.chain[0];
    // let header = jsonwebtoken::decode_header(&client_token).unwrap();
    // let public_key = get_public_key(header.x5u.unwrap().as_bytes()).unwrap();
    //
    // for token in tokens.chain {
    //     let split = token.split('.');
    //     for part in split {
    //         if let Ok(decoded) = ENGINE.decode(part) {
    //             tracing::info!("{}", String::from_utf8_lossy(&decoded).to_string());
    //         }
    //     }
    // }

    todo!();

    Ok(())
}

// fn decode_identity_data(buffer: &mut BytesMut) -> VexResult<TokenClaims> {
//     // TODO: Verify Mojang and Xbox public keys
//
//     let token_length = buffer.get_u32_le();
//     let position = buffer.len() - buffer.remaining();
//     let token = &buffer.as_ref()[position..(position + token_length as usize)];
//
//     let chains = match serde_json::from_slice::<TokenChain>(token) {
//         Ok(c) => c,
//         Err(e) => {
//             return Err(vex_error!(InvalidRequest, format!("Invalid JSON: {e}")));
//         }
//     };
//
//     let base64_engine = base64::engine::general_purpose::STANDARD_NO_PAD;
//     for (index, chain) in chains.chain.iter().enumerate() {
//         let mut split = chain.split('.');
//         if let Some(second) = split.nth(1) {
//             if let Ok(decoded) = base64_engine.decode(second) {
//                 if let Ok(json) = serde_json::from_slice(&decoded) {
//                     return Ok(json);
//                 }
//             }
//         }
//     }
//
//     Err(vex_error!(
//         InvalidRequest,
//         "No identity data was found in the login tokens"
//     ))
// }
