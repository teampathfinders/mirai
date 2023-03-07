use std::io::Write;

use base64::Engine;
use bytes::{Buf, Bytes, BytesMut};
use common::{bail, error, VResult};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use p384::pkcs8::spki;
use uuid::Uuid;

use crate::network::packets::login::{DeviceOS, UiProfile};
use crate::network::Skin;

/// Mojang's public key.
/// Used to verify the second token in the identity chain.
pub const MOJANG_PUBLIC_KEY: &str = "MHYwEAYHKoZIzj0CAQYFK4EEACIDYgAE8ELkixyLcwlZryUQcu1TvPOmI2B7vX83ndnWRUaXm74wFfa5f/lwQNTfrLVHa2PmenpGI6JhIMUJaWZrjmMj90NoKNFSNBuKdm8rYiXsfaz3K36x/1U26HpG0ZxK/V1V";

/// Use the default Base64 format with no padding.
const BASE64_ENGINE: base64::engine::GeneralPurpose =
    base64::engine::general_purpose::STANDARD_NO_PAD;

/// Data contained in the identity token chain.
#[derive(Debug, Clone)]
pub struct IdentityData {
    /// Xbox account ID.
    pub xuid: u64,
    /// UUID unique for this player.
    pub uuid: Uuid,
    /// Xbox username.
    pub display_name: String,
    /// Public key used for token verification and encryption.
    pub public_key: String,
}

/// A chain of JSON web tokens.
#[derive(serde::Deserialize, Debug)]
pub struct TokenChain {
    /// Chain of JWTs.
    pub chain: Vec<String>,
}

/// Used to extract the public key from the identity tokens.
#[derive(serde::Deserialize, Debug)]
pub struct KeyTokenPayload {
    #[serde(rename = "identityPublicKey")]
    pub public_key: String,
}

/// Data extracted from the "extraData" field in the last token in the identity chain.
#[derive(serde::Deserialize, Debug)]
pub struct RawIdentityData {
    #[serde(rename = "XUID")]
    pub xuid: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "identity")]
    pub uuid: Uuid,
}

/// Used to extract the identity data and public key from the last identity token.
#[derive(serde::Deserialize, Debug)]
pub struct IdentityTokenPayload {
    #[serde(rename = "extraData")]
    pub client_data: RawIdentityData,
    #[serde(rename = "identityPublicKey")]
    pub public_key: String,
}

/// Used to extract data from the user data token.
#[derive(serde::Deserialize, Debug, Clone)]
pub struct UserData {
    /// Operating system of the client.
    #[serde(rename = "DeviceOS")]
    pub build_platform: DeviceOS,
    #[serde(rename = "DeviceModel")]
    pub device_model: String,
    #[serde(rename = "DeviceId")]
    pub device_id: String,
    /// Language in ISO format (i.e. en_GB)
    #[serde(rename = "LanguageCode")]
    pub language_code: String,
    #[serde(rename = "UIProfile")]
    pub ui_profile: UiProfile,
    #[serde(rename = "GuiScale")]
    pub gui_scale: i32,
}

#[derive(serde::Deserialize, Debug)]
pub struct UserDataTokenPayload {
    #[serde(flatten)]
    pub data: UserData,
    #[serde(flatten)]
    pub skin: Skin,
}

/// First token in the chain holds the client's self-signed public key in the X5U.
/// It is extracted from the header of the token and used to verify its signature.
/// The payload of the token contains a new key which is used to verify the next token.
fn parse_initial_token(token: &str) -> VResult<String> {
    // Decode JWT header to get X5U.
    let header = jsonwebtoken::decode_header(token)?;
    let base64 = header.x5u.ok_or_else(|| {
        error!(BadPacket, "Missing X.509 certificate URL (x5u)")
    })?;

    let bytes = BASE64_ENGINE.decode(base64)?;

    // Public key that can be used to verify the token.
    let public_key = match spki::SubjectPublicKeyInfoRef::try_from(bytes.as_ref())
    {
        Ok(p) => p,
        Err(e) => bail!(InvalidIdentity, "Invalid client public key: {e}"),
    };

    let decoding_key = DecodingKey::from_ec_der(public_key.subject_public_key.raw_bytes());
    let mut validation = Validation::new(Algorithm::ES384);
    validation.validate_exp = true;
    validation.validate_nbf = true;

    let payload = jsonwebtoken::decode::<KeyTokenPayload>(
        token,
        &decoding_key,
        &validation,
    )?;
    Ok(payload.claims.public_key)
}

/// The second token in the chain can be verified using Mojang's public key
/// (or the identityPublicKey from the previous token).
/// This token contains another identityPublicKey which is the public key for the third token.
fn parse_mojang_token(token: &str, key: &str) -> VResult<String> {
    let bytes = BASE64_ENGINE.decode(key)?;
    let public_key = match spki::SubjectPublicKeyInfoRef::try_from(bytes.as_ref())
    {
        Ok(p) => p,
        Err(e) => bail!(InvalidIdentity, "Invalid client public key: {e}"),
    };

    let decoding_key = DecodingKey::from_ec_der(public_key.subject_public_key.raw_bytes());
    let mut validation = Validation::new(Algorithm::ES384);
    validation.set_issuer(&["Mojang"]);
    validation.validate_nbf = true;
    validation.validate_exp = true;

    let payload = jsonwebtoken::decode::<KeyTokenPayload>(
        token,
        &decoding_key,
        &validation,
    )?;

    Ok(payload.claims.public_key)
}

/// The third token contains the client's actual public key and extra data.
/// The extraData field contains the XUID, client identity (UUID) and the display name.
///
/// Just like the second one, this token can be verified using the identityPublicKey from the last token.
fn parse_identity_token(
    token: &str,
    key: &str,
) -> VResult<IdentityTokenPayload> {
    let bytes = BASE64_ENGINE.decode(key)?;
    let public_key = match spki::SubjectPublicKeyInfoRef::try_from(
        bytes.as_ref()
    )
    {
        Ok(p) => p,
        Err(e) => bail!(InvalidIdentity, "Invalid client public key: {e}"),
    };

    let decoding_key = DecodingKey::from_ec_der(public_key.subject_public_key.raw_bytes());
    let mut validation = Validation::new(Algorithm::ES384);
    validation.set_issuer(&["Mojang"]);
    validation.validate_nbf = true;
    validation.validate_exp = true;

    let payload = jsonwebtoken::decode::<IdentityTokenPayload>(
        token,
        &decoding_key,
        &validation,
    )?;

    Ok(payload.claims)
}

/// Verifies and decodes the user data token.
fn parse_user_data_token(
    token: &str,
    key: &str,
) -> VResult<UserDataTokenPayload> {
    let bytes = BASE64_ENGINE.decode(key)?;
    let public_key = match spki::SubjectPublicKeyInfoRef::try_from(bytes.as_ref())
    {
        Ok(p) => p,
        Err(e) => bail!(InvalidIdentity, "Invalid client public key: {e}"),
    };

    let decoding_key = DecodingKey::from_ec_der(public_key.subject_public_key.raw_bytes());
    let mut validation = Validation::new(Algorithm::ES384);

    // No special header data included in this token, don't verify anything.
    validation.required_spec_claims.clear();

    let payload = jsonwebtoken::decode::<UserDataTokenPayload>(
        token,
        &decoding_key,
        &validation,
    )?;

    Ok(payload.claims)
}

/// Parses the identification data contained in the first token chain.
///
/// This contains such as the XUID, display name and public key.
pub fn parse_identity_data(
    buffer: &mut Bytes,
) -> VResult<IdentityTokenPayload> {
    let token_length = buffer.get_u32_le();
    let position = buffer.len() - buffer.remaining();
    let token_chain =
        &buffer.as_ref()[position..(position + token_length as usize)];

    let tokens = serde_json::from_slice::<TokenChain>(token_chain)?;
    buffer.advance(token_length as usize);

    let identity_data = match tokens.chain.len() {
        1 => {
            // Client is not signed into Xbox.
            bail!(
                NotAuthenticated,
                "User must be authenticated with Microsoft services."
            );
        }
        3 => {
            // Verify the first token and decode the public key for the next token.
            // This public key must be equal to Mojang's public key to verify that the second
            // token was signed by Mojang.
            let mut key = parse_initial_token(&tokens.chain[0])?;
            if !key.eq(MOJANG_PUBLIC_KEY) {
                bail!(
                    InvalidIdentity,
                    "Identity token was not signed by Mojang"
                );
            }

            key = parse_mojang_token(&tokens.chain[1], &key)?;
            parse_identity_token(&tokens.chain[2], &key)?
        }
        _ => bail!(
            BadPacket,
            "Unexpected token count {}, expected 3",
            tokens.chain.len()
        ),
    };

    Ok(identity_data)
}

/// Parses the user data token from the login packet.
/// This token contains the user's operating system, language, skin, etc.
pub fn parse_user_data(
    buffer: &mut Bytes,
    public_key: &str,
) -> VResult<UserDataTokenPayload> {
    let token_length = buffer.get_u32_le();
    let position = buffer.len() - buffer.remaining();
    let token = &buffer.as_ref()[position..(position + token_length as usize)];
    let token_string = String::from_utf8_lossy(token);

    let user_data = parse_user_data_token(token_string.as_ref(), public_key)?;

    Ok(user_data)
}
