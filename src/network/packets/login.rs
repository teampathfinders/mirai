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

#[derive(serde::Deserialize, Debug)]
pub struct KeyTokenPayload {
    #[serde(rename = "identityPublicKey")]
    pub public_key: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct IdentityData {
    #[serde(rename = "XUID")]
    pub xuid: String,

    #[serde(rename = "displayName")]
    pub display_name: String,

    #[serde(rename = "identity")]
    pub uuid: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct IdentityTokenPayload {
    #[serde(rename = "extraData")]
    pub client_data: IdentityData,
    #[serde(rename = "identityPublicKey")]
    pub public_key: String,
}

/// First token in the chain holds the client's self-signed public key in the X5U.
/// It is extracted from the header of the token and used to verify its signature.
/// The payload of the token contains a new key which is used to verify the next token.
fn verify_first_token(token: &str) -> VexResult<String> {
    // Decode JWT header to get X5U.
    let header = jsonwebtoken::decode_header(token)?;
    let base64 = header.x5u.ok_or(
        error!(InvalidRequest, "Missing X.509 certificate URL in token")
    )?;
    let bytes = ENGINE.decode(base64)?;

    // Public key that can be used to verify the token.
    let public_key = spki::SubjectPublicKeyInfo::try_from(bytes.as_ref())?;
    let decoding_key = DecodingKey::from_ec_der(public_key.subject_public_key);
    let mut validation = Validation::new(Algorithm::ES384);
    validation.validate_exp = true;
    validation.validate_nbf = true;

    let payload = jsonwebtoken::decode::<KeyTokenPayload>(token, &decoding_key, &validation)?;
    tracing::info!("{payload:?}");

    Ok(payload.claims.public_key)
}

/// The second token in the chain can be verified using Mojang's public key
/// (or the identityPublicKey from the previous token).
/// This token contains another identityPublicKey which is the public key for the third token.
fn verify_second_token(token: &str, key: &str) -> VexResult<String> {
    let bytes = ENGINE.decode(key)?;
    let public_key = spki::SubjectPublicKeyInfo::try_from(bytes.as_ref())?;
    let decoding_key = DecodingKey::from_ec_der(public_key.subject_public_key);
    let mut validation = Validation::new(Algorithm::ES384);
    validation.set_issuer(&["Mojang"]);
    validation.validate_nbf = true;
    validation.validate_exp = true;

    let payload = jsonwebtoken::decode::<KeyTokenPayload>(token, &decoding_key, &validation)?;
    tracing::info!("{payload:?}");

    Ok(payload.claims.public_key)
}

fn verify_third_token(token: &str, key: &str) -> VexResult<IdentityTokenPayload> {
    let bytes = ENGINE.decode(key)?;
    let public_key = spki::SubjectPublicKeyInfo::try_from(bytes.as_ref())?;
    let decoding_key = DecodingKey::from_ec_der(public_key.subject_public_key);
    let mut validation = Validation::new(Algorithm::ES384);
    validation.set_issuer(&["Mojang"]);
    validation.validate_nbf = true;
    validation.validate_exp = true;

    let payload = jsonwebtoken::decode::<IdentityTokenPayload>(token, &decoding_key, &validation)?;
    tracing::info!("{payload:?}");

    Ok(payload.claims)
}

fn parse_identity_data(buffer: &mut BytesMut) -> VexResult<()> {
    let token_length = buffer.get_u32_le();
    let position = buffer.len() - buffer.remaining();
    let token = &buffer.as_ref()[position..(position + token_length as usize)];

    let tokens = serde_json::from_slice::<TokenChain>(token)?;
    if tokens.chain.is_empty() {
        bail!(InvalidRequest, format!("Client sent {} tokens, expected 3", tokens.chain.len()));
    }
    tracing::info!("{tokens:?}");

    match tokens.chain.len() {
        1 => {
            // Client is not signed into Xbox.
            bail!(InvalidRequest, "Client is not signed into Xbox");
        },
        3 => {
            // Verify the first token and decode the public key for the next token.
            // This public key must be equal to Mojang's public key to verify that the second
            // token was signed by Mojang.
            let mut key = verify_first_token(&tokens.chain[0])?;
            if !key.eq(MOJANG_PUBLIC_KEY) {
                bail!(InvalidRequest, "Token was not signed by Mojang");
            }

            key = verify_second_token(&tokens.chain[1], &key)?;

            let identity_data = verify_third_token(&tokens.chain[2], &key)?;
        },
        _ => bail!(InvalidRequest, "Unexpected token count")
    }

    todo!();

    // let client_token = jsonwebtoken::decode_header(&tokens.chain[0])?;
    // let raw_client_key = client_token.x5u.ok_or(error!(InvalidRequest, "Missing x5u in client-signed token"))?;
    // let decoded_raw = ENGINE.decode(raw_client_key).unwrap();
    // let info = spki::SubjectPublicKeyInfo::try_from(decoded_raw.as_ref()).unwrap();
    // tracing::debug!("{info:?}");
    //
    // let decoding_key = DecodingKey::from_ec_der(info.subject_public_key);
    // let validation = Validation::new(Algorithm::ES384);
    // let client_token = jsonwebtoken::decode::<KeyTokenPayload>(&tokens.chain[0], &decoding_key, &validation).unwrap();
    // tracing::info!("{client_token:?}");

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
