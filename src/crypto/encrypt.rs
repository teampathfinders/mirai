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

const ENGINE: base64::engine::GeneralPurpose = base64::engine::general_purpose::STANDARD;

#[derive(serde::Serialize, Debug)]
struct EncryptionTokenPayload<'a> {
    salt: &'a str,
    // exp: u64,
    // nbf: u64,
    // #[serde(rename = "identityPublicKey")]
    // identity_public_key: &'a str
}

pub struct EncryptionData {
    pub jwt: String,
    // pub private_key: SigningKey<NistP384>
}

pub fn perform_key_exchange(client_raw_public_key: &str) -> VexResult<EncryptionData> {
    let salt = (0..16).map(|_| OsRng.sample(Alphanumeric) as char).collect::<String>();

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
        salt: &ENGINE.encode(salt)
        // salt: &salt
    };

    let jwt = jsonwebtoken::encode(&header, &claims, &encoding_key).unwrap();
    tracing::info!("{jwt:?}");

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

    Ok(EncryptionData {
        jwt,

    })
}

// /// Perform the Diffie-Hellman key exchange
// pub fn perform_key_exchange(client_x509: &str) -> VexResult<EncryptionData> {
//     // let salt = (0..16).map(|_| OsRng.sample(Alphanumeric) as char).collect::<String>();
//     let salt = "🧂";
//
//     let raw_key = ENGINE.decode(client_x509)?;
//     let client_public_key = PublicKey::from_public_key_der(&raw_key)?;
//
//     let server_secret = SecretKey::random(OsRng);
//     let secret_der = server_secret.to_pkcs8_der().unwrap();
//
//     let server_public_key = server_secret.public_key();
//     let server_public_der = {
//         let bytes = server_public_key.to_public_key_der().unwrap();
//         ENGINE.encode(bytes)
//     };
//
//     // let shared_secret = diffie_hellman(
//     //     server_secret.to_nonzero_scalar(),
//     //     client_public_key.as_affine(),
//     // );
//     // let secret_hash = shared_secret.extract::<sha2::Sha256>(Some(SALT.as_bytes()));
//     //
//     // let mut okm = [0u8; 32];
//     // secret_hash.expand(&[], &mut okm).unwrap();
//
//     let mut header = jsonwebtoken::Header::new(Algorithm::ES384);
//     header.x5u = Some(server_public_der.clone());
//
//     let payload = EncryptionTokenPayload {
//         salt: &ENGINE.encode(salt),
//         nbf: 1674748494,
//         exp: 1674921354,
//         identity_public_key: &server_public_der
//     };
//
//     let encoding_key = EncodingKey::from_ec_der(&secret_der.to_bytes());
//     let jwt = jsonwebtoken::encode(&header, &payload, &encoding_key).unwrap();
//
//     {
//         #[derive(serde::Deserialize, Debug)]
//         struct TestPayload {
//             salt: String
//         }
//
//         let bytes = ENGINE.decode(server_public_der)?;
//         // Public key that can be used to verify the token.
//         let public_key = spki::SubjectPublicKeyInfo::try_from(bytes.as_ref())?;
//
//         let decoding_key = DecodingKey::from_ec_der(public_key.subject_public_key);
//         let mut validation = Validation::new(Algorithm::ES384);
//         validation.validate_exp = true;
//         validation.validate_nbf = true;
//
//         let payload = jsonwebtoken::decode::<TestPayload>(&jwt, &decoding_key, &validation)?;
//         tracing::info!("{payload:?}");
//     }
//
//     tracing::info!("{jwt}");
//
//     Ok(EncryptionData {
//         jwt
//     })
// }

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