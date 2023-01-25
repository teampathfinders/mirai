use base64::Engine;
use bytes::{Buf, BytesMut};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde_derive::Deserialize;
use spki::SubjectPublicKeyInfo;

use crate::{bail, error, vex_assert};
use crate::error::{VexError, VexResult};
use crate::network::packets::GamePacket;
use crate::network::traits::Decodable;
use crate::util::ReadExtensions;

pub const MOJANG_PUBLIC_KEY: &str = "MHYwEAYHKoZIzj0CAQYFK4EEACIDYgAE8ELkixyLcwlZryUQcu1TvPOmI2B7vX83ndnWRUaXm74wFfa5f/lwQNTfrLVHa2PmenpGI6JhIMUJaWZrjmMj90NoKNFSNBuKdm8rYiXsfaz3K36x/1U26HpG0ZxK/V1V";
const ENGINE: base64::engine::GeneralPurpose = base64::engine::general_purpose::STANDARD_NO_PAD;

#[derive(Debug)]
pub enum DeviceOS {
    Android,
    IOS,
    OSX,
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
            2 => IOS,
            3 => OSX,
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
            _ => bail!(InvalidRequest, "Invalid device OS")
        })
    }
}

#[derive(Debug)]
pub struct IdentityData {
    pub xuid: u64,
    pub identity: String,
    pub display_name: String,
    pub public_key: String,
}

#[derive(Debug)]
pub struct UserData {
    pub device_os: DeviceOS,
    pub language_code: String,
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

#[derive(serde::Deserialize, Debug)]
pub struct TokenChain {
    pub chain: Vec<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct KeyTokenPayload {
    #[serde(rename = "identityPublicKey")]
    pub public_key: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct RawIdentityData {
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
    pub client_data: RawIdentityData,
    #[serde(rename = "identityPublicKey")]
    pub public_key: String,
}

/// First token in the chain holds the client's self-signed public key in the X5U.
/// It is extracted from the header of the token and used to verify its signature.
/// The payload of the token contains a new key which is used to verify the next token.
fn verify_first_token(token: &str) -> VexResult<String> {
    // Decode JWT header to get X5U.
    let header = jsonwebtoken::decode_header(token)?;
    let base64 = header.x5u.ok_or(error!(
        InvalidRequest,
        "Missing X.509 certificate URL in token"
    ))?;
    let bytes = ENGINE.decode(base64)?;
    // Public key that can be used to verify the token.
    let public_key = spki::SubjectPublicKeyInfo::try_from(bytes.as_ref())?;

    let decoding_key = DecodingKey::from_ec_der(public_key.subject_public_key);
    let mut validation = Validation::new(Algorithm::ES384);
    validation.validate_exp = true;
    validation.validate_nbf = true;

    let payload = jsonwebtoken::decode::<KeyTokenPayload>(token, &decoding_key, &validation)?;
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
    Ok(payload.claims.public_key)
}

/// The third token contains the client's actual public key and extra data.
/// The extraData field contains the XUID, client identity (UUID) and the display name.
///
/// Just like the second one, this token can be verified using the identityPublicKey from the last token.
fn verify_third_token(token: &str, key: &str) -> VexResult<IdentityTokenPayload> {
    let bytes = ENGINE.decode(key)?;
    let public_key = spki::SubjectPublicKeyInfo::try_from(bytes.as_ref())?;

    let decoding_key = DecodingKey::from_ec_der(public_key.subject_public_key);
    let mut validation = Validation::new(Algorithm::ES384);
    validation.set_issuer(&["Mojang"]);
    validation.validate_nbf = true;
    validation.validate_exp = true;

    let payload = jsonwebtoken::decode::<IdentityTokenPayload>(token, &decoding_key, &validation)?;
    Ok(payload.claims)
}

#[derive(serde::Deserialize, Debug)]
struct UserTokenPayload {
    #[serde(rename = "DeviceOS")]
    device_os: u8,
    #[serde(rename = "LanguageCode")]
    language_code: String,
    #[serde(rename = "ServerAddress")]
    server_address: String,
}

fn verify_fourth_token(token: &str, key: &str) -> VexResult<UserTokenPayload> {
    let bytes = ENGINE.decode(key)?;
    let public_key = spki::SubjectPublicKeyInfo::try_from(bytes.as_ref())?;

    let decoding_key = DecodingKey::from_ec_der(public_key.subject_public_key);
    let mut validation = Validation::new(Algorithm::ES384);

    // No special header data included in this token, don't verify anything.
    validation.required_spec_claims.clear();

    let payload = jsonwebtoken::decode::<UserTokenPayload>(token, &decoding_key, &validation)?;
    Ok(payload.claims)
}

fn parse_identity_data(buffer: &mut BytesMut) -> VexResult<IdentityTokenPayload> {
    let token_length = buffer.get_u32_le();
    let position = buffer.len() - buffer.remaining();
    let token = &buffer.as_ref()[position..(position + token_length as usize)];

    let tokens = serde_json::from_slice::<TokenChain>(token)?;
    if tokens.chain.is_empty() {
        bail!(
            InvalidRequest,
            format!("Client sent {} tokens, expected 3", tokens.chain.len())
        );
    }
    buffer.advance(token_length as usize);

    let identity_data = match tokens.chain.len() {
        1 => {
            // Client is not signed into Xbox.
            bail!(InvalidRequest, "Client is not signed into Xbox");
        }
        3 => {
            // Verify the first token and decode the public key for the next token.
            // This public key must be equal to Mojang's public key to verify that the second
            // token was signed by Mojang.
            let mut key = verify_first_token(&tokens.chain[0])?;
            if !key.eq(MOJANG_PUBLIC_KEY) {
                bail!(InvalidRequest, "Token was not signed by Mojang");
            }

            key = verify_second_token(&tokens.chain[1], &key)?;
            verify_third_token(&tokens.chain[2], &key)?
        }
        _ => bail!(InvalidRequest, "Unexpected token count"),
    };

    Ok(identity_data)
}

fn parse_user_data(buffer: &mut BytesMut, public_key: &str) -> VexResult<UserTokenPayload> {
    let token_length = buffer.get_u32_le();
    let position = buffer.len() - buffer.remaining();
    let token = &buffer.as_ref()[position..(position + token_length as usize)];
    let token_string = String::from_utf8_lossy(token);

    let user_data = verify_fourth_token(token_string.as_ref(), public_key)?;

    Ok(user_data)
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
