use base64::Engine;
use bytes::{Buf, BytesMut};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};

use crate::error::VexResult;
use crate::network::packets::DeviceOS;
use crate::{bail, error};

pub const MOJANG_PUBLIC_KEY: &str = "MHYwEAYHKoZIzj0CAQYFK4EEACIDYgAE8ELkixyLcwlZryUQcu1TvPOmI2B7vX83ndnWRUaXm74wFfa5f/lwQNTfrLVHa2PmenpGI6JhIMUJaWZrjmMj90NoKNFSNBuKdm8rYiXsfaz3K36x/1U26HpG0ZxK/V1V";
const BASE64_ENGINE: base64::engine::GeneralPurpose =
    base64::engine::general_purpose::STANDARD_NO_PAD;

#[derive(Debug)]
pub struct IdentityData {
    pub xuid: u64,
    pub identity: String,
    pub display_name: String,
    pub title_id: u32,
    pub public_key: String,
}

#[derive(Debug)]
pub struct UserData {
    pub device_os: DeviceOS,
    pub language_code: String,
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
    #[serde(rename = "titleId")]
    pub title_id: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct IdentityTokenPayload {
    #[serde(rename = "extraData")]
    pub client_data: RawIdentityData,
    #[serde(rename = "identityPublicKey")]
    pub public_key: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct UserTokenPayload {
    #[serde(rename = "DeviceOS")]
    pub device_os: u8,
    #[serde(rename = "LanguageCode")]
    pub language_code: String,
    #[serde(rename = "ServerAddress")]
    server_address: String,
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
    let bytes = BASE64_ENGINE.decode(base64)?;
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
    let bytes = BASE64_ENGINE.decode(key)?;
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
    let bytes = BASE64_ENGINE.decode(key)?;
    let public_key = spki::SubjectPublicKeyInfo::try_from(bytes.as_ref())?;

    let decoding_key = DecodingKey::from_ec_der(public_key.subject_public_key);
    let mut validation = Validation::new(Algorithm::ES384);
    validation.set_issuer(&["Mojang"]);
    validation.validate_nbf = true;
    validation.validate_exp = true;

    let payload = jsonwebtoken::decode::<IdentityTokenPayload>(token, &decoding_key, &validation)?;
    Ok(payload.claims)
}

fn verify_fourth_token(token: &str, key: &str) -> VexResult<UserTokenPayload> {
    let bytes = BASE64_ENGINE.decode(key)?;
    let public_key = spki::SubjectPublicKeyInfo::try_from(bytes.as_ref())?;

    let decoding_key = DecodingKey::from_ec_der(public_key.subject_public_key);
    let mut validation = Validation::new(Algorithm::ES384);

    // No special header data included in this token, don't verify anything.
    validation.required_spec_claims.clear();

    let payload = jsonwebtoken::decode::<UserTokenPayload>(token, &decoding_key, &validation)?;
    Ok(payload.claims)
}

pub fn parse_identity_data(buffer: &mut BytesMut) -> VexResult<IdentityTokenPayload> {
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

pub fn parse_user_data(buffer: &mut BytesMut, public_key: &str) -> VexResult<UserTokenPayload> {
    let token_length = buffer.get_u32_le();
    let position = buffer.len() - buffer.remaining();
    let token = &buffer.as_ref()[position..(position + token_length as usize)];
    let token_string = String::from_utf8_lossy(token);

    let user_data = verify_fourth_token(token_string.as_ref(), public_key)?;

    Ok(user_data)
}
