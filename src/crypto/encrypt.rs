use std::fmt::{Debug, Formatter, Write};

use anyhow::bail;
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

/// Use the default Base64 format with no padding.
const BASE64_ENGINE: base64::engine::GeneralPurpose = base64::engine::general_purpose::STANDARD_NO_PAD;

/// Payload of the encryption handshake token
#[derive(serde::Serialize, Debug)]
struct EncryptionTokenClaims<'a> {
    salt: &'a str,
}

/// Used to encrypt and decrypt packets with AES.
pub struct Encryptor {
    secret: [u8; 16],
}

impl Debug for Encryptor {
    /// Allow usage in debug derived structs, but prevent logging the secret.
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        fmt.write_str("Encryptor")
    }
}

impl Encryptor {
    /// Creates a new encryptor.
    ///
    /// This generates a unique private and public key pair for the current session.
    /// A JWT containing the public key and salt is returned.
    /// The public key is contained in the x5u header field and
    /// the salt is contained in the payload.
    /// The JWT is signed using the session's private key.
    ///
    /// Besides creating the JWT, a Diffie-Hellman key exchange is also performed.
    /// The same key exchange is executed on the client's side,
    /// this produces the same shared secret on both sides, without knowing each other's private keys.
    ///
    /// This shared secret is hashed using with SHA-256 and the salt contained in the JWT.
    /// The produced hash can then be used to encrypt packets.
    pub fn new(client_public_key_der: &str) -> anyhow::Result<(Self, String)> {
        // Generate a random salt using a cryptographically secure generator.
        let salt = (0..16)
            .map(|_| OsRng.sample(Alphanumeric) as char)
            .collect::<String>();

        // Generate a random private key for the session.
        let private_key: SigningKey<NistP384> = ecdsa::SigningKey::random(&mut OsRng);
        // Convert the key to the PKCS#8 DER format used by Minecraft.
        let private_key_der = match private_key.to_pkcs8_der() {
            Ok(k) => k,
            Err(e) => bail!("Failed to convert private to PKCS#8 DER format: {e}")
        };

        // Extract and convert the public key, which will be sent to the client.
        let public_key = private_key.verifying_key();
        let public_key_der = {
            let binary_der = match public_key.to_public_key_der() {
                Ok(d) => d,
                Err(e) => bail!("Failed to convert public key to DER format: {e}")
            };
            BASE64_ENGINE.encode(binary_der)
        };

        // The typ header is set to none to match the official server software.
        let mut header = jsonwebtoken::Header::new(Algorithm::ES384);
        header.typ = None;
        header.x5u = Some(public_key_der);

        // Create a JWT encoding key using the session's private key.
        let signing_key = jsonwebtoken::EncodingKey::from_ec_der(&private_key_der.to_bytes());
        let claims = EncryptionTokenClaims {
            salt: &BASE64_ENGINE.encode(&salt)
        };

        let jwt = jsonwebtoken::encode(&header, &claims, &signing_key)?;
        let client_public_key = {
            let bytes = BASE64_ENGINE.decode(client_public_key_der)?;
            match PublicKey::from_public_key_der(&bytes) {
                Ok(k) => k,
                Err(e) => bail!("Failed to read DER-encoded client public key: {e}")
            }
        };

        // Perform the key exchange
        let shared_secret = diffie_hellman(
            private_key.as_nonzero_scalar(),
            client_public_key.as_affine(),
        );
        let secret_hash = shared_secret.extract::<sha2::Sha256>(Some(salt.as_bytes()));

        let mut okm = [0u8; 32];
        match secret_hash.expand(&[], &mut okm) {
            Ok(h) => h,
            Err(e) => bail!("Failed to expand shared secret hash: {e}")
        }

        /// Minecraft uses the first 16 bytes of the hash for encryption.
        let mut secret = [0u8; 16];
        secret.copy_from_slice(&okm[..16]);

        Ok((Self {
            secret
        }, jwt))
    }
}