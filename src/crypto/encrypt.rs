use base64::Engine;
use ecdsa::elliptic_curve::pkcs8::EncodePrivateKey;
use ecdsa::SigningKey;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Validation};
use lazy_static::lazy_static;
use p384::{NistP384, SecretKey};
use p384::ecdh::diffie_hellman;
use p384::PublicKey;
use rand::distributions::Alphanumeric;
use rand::Rng;
use rand::rngs::OsRng;
use spki::{DecodePublicKey, EncodePublicKey};
use spki::der::SecretDocument;

use crate::error::VexResult;

const ENGINE: base64::engine::GeneralPurpose = base64::engine::general_purpose::STANDARD_NO_PAD;

#[derive(serde::Serialize, Debug)]
struct EncryptionTokenPayload<'a> {
    salt: &'a str
}

pub struct EncryptionData {
    pub jwt: String,
    // pub private_key: SigningKey<NistP384>
}

pub fn perform_key_exchange(client_raw_public_key: &str) -> VexResult<EncryptionData> {
    let salt = (0..16)
        .map(|_| OsRng.sample(Alphanumeric) as char)
        .collect::<String>();

    let private_key: SigningKey<NistP384> = ecdsa::SigningKey::random(&mut OsRng);
    let der_private_key = private_key.to_pkcs8_der().unwrap();

    let public_key = private_key.verifying_key();
    let der_public_key = {
        let der_key = public_key.to_public_key_der().unwrap();

        let base64 = ENGINE.encode(der_key.as_bytes());
        base64
    };

    let mut header = jsonwebtoken::Header::new(Algorithm::ES384);
    header.typ = None;
    header.x5u = Some(der_public_key);

    let encoding_key = jsonwebtoken::EncodingKey::from_ec_der(&der_private_key.to_bytes());
    let claims = EncryptionTokenPayload {
        salt: &ENGINE.encode(salt),
    };

    let jwt = jsonwebtoken::encode(&header, &claims, &encoding_key).unwrap();

    // let client_public_key = {
    //     let bytes = ENGINE.decode(client_raw_public_key).unwrap();
    //     PublicKey::from_public_key_der(&bytes).unwrap()
    // };
    //
    // let shared_secret = diffie_hellman(
    //     private_key.as_nonzero_scalar(),
    //     client_public_key.as_affine(),
    // );
    // let secret_hash = shared_secret.extract::<sha2::Sha256>(Some(salt.as_bytes()));
    //
    // let mut okm = [0u8; 32];
    // secret_hash.expand(&[], &mut okm).unwrap();

    Ok(EncryptionData { jwt })
}