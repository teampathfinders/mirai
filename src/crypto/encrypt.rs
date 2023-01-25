use base64::Engine;
use jsonwebtoken::{Algorithm, EncodingKey};
use lazy_static::lazy_static;
use p384::ecdh::{diffie_hellman, EphemeralSecret};
use p384::elliptic_curve::rand_core::OsRng;
use p384::{EncodedPoint, PublicKey, SecretKey};
use rand::distributions::Alphanumeric;
use spki::DecodePublicKey;
use crate::error::VexResult;
use rand::Rng;

const ENGINE: base64::engine::GeneralPurpose = base64::engine::general_purpose::STANDARD_NO_PAD;

lazy_static! {
    pub static ref SALT: String = {
        let mut rng = rand::thread_rng();
        (0..8).map(|_| rng.sample(Alphanumeric) as char).collect::<String>()
    };
}

#[derive(serde::Serialize, Debug)]
struct EncryptionTokenPayload<'a> {
    salt: &'a str,
    #[serde(rename = "signedToken")]
    signed_token: &'a str,
}

/// Perform the Diffie-Hellman key exchange
pub fn key_exchange(client_x509: &str) -> VexResult<()> {
    let raw_key = ENGINE.decode(client_x509)?;
    let client_public_key = PublicKey::from_public_key_der(&raw_key)?;

    let server_secret = SecretKey::random(OsRng);
    let secret_der = server_secret.to_sec1_der().unwrap();

    let shared_secret = diffie_hellman(
        server_secret.to_nonzero_scalar(),
        client_public_key.as_affine(),
    );
    let secret_hash = shared_secret.extract::<sha2::Sha256>(Some(SALT.as_bytes()));

    let mut okm = [0u8; 32];
    secret_hash.expand(&[], &mut okm).unwrap();

    let mut header = jsonwebtoken::Header::new(Algorithm::ES384);
    header.x5u = Some(client_x509.to_string());

    let payload = EncryptionTokenPayload {
        salt: SALT.as_str(),
        signed_token: client_x509,
    };

    let encoding_key = EncodingKey::from_ec_der(&secret_der);
    let jwt = jsonwebtoken::encode(&header, &payload, &encoding_key).unwrap();

    tracing::info!("{jwt:?}");

    Ok(())
}

// pub fn key_exchange(client_x509: &str) -> VexResult<()> {
//     let raw_key = ENGINE.decode(client_x509)?;
//     let client_public_key = PublicKey::from_public_key_der(&raw_key)?;
//
//     let server_secret = EphemeralSecret::random(&mut OsRng);
//
//     // let public_key = EncodedPoint::from(server_secret.public_key());
//     let shared_secret = server_secret.diffie_hellman(&client_public_key);
//     let secret_hash = shared_secret.extract::<sha2::Sha256>(Some(SALT.as_bytes()));
//
//     let mut okm = [0u8; 32];
//     secret_hash.expand(&[], &mut okm).unwrap();
//
//     let mut header = jsonwebtoken::Header::new(Algorithm::ES384);
//     header.x5u = Some(client_x509.to_string());
//
//     let payload = EncryptionTokenPayload {
//         salt: SALT.as_str(),
//         signed_token: client_x509
//     };
//
//     let encoding_key = EncodingKey::from_ec_der(okm);
//     let jwt = jsonwebtoken::encode(&header, &payload, &encoding_key)?;
//
//     Ok(())
// }