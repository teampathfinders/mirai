use std::fmt::{Debug, Formatter, Write};
use std::io::Read;
use std::sync::atomic::{AtomicU16, AtomicU32, AtomicU64, Ordering};

use anyhow::{bail, Context};
use base64::Engine;
use bytes::{BufMut, BytesMut};
use cipher::StreamCipher;
use ctr::cipher::KeyIvInit;
use ecdsa::elliptic_curve::pkcs8::EncodePrivateKey;
use ecdsa::SigningKey;
use flate2::read::DeflateDecoder;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Validation};
use lazy_static::lazy_static;
use p384::{NistP384, SecretKey};
use p384::ecdh::{diffie_hellman, SharedSecret};
use p384::PublicKey;
use parking_lot::Mutex;
use rand::distributions::Alphanumeric;
use rand::Rng;
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};
use spki::{DecodePublicKey, EncodePublicKey};
use spki::der::SecretDocument;

type Aes256Ctr64LE = ctr::Ctr64LE<aes::Aes256>;

/// Use the default Base64 format with no padding.
const BASE64_ENGINE: base64::engine::GeneralPurpose = base64::engine::general_purpose::STANDARD_NO_PAD;

/// Payload of the encryption handshake token
#[derive(serde::Serialize, Debug)]
struct EncryptionTokenClaims<'a> {
    salt: &'a str,
}

/// Used to encrypt and decrypt packets with AES.
pub struct Encryptor {
    cipher: Mutex<Aes256Ctr64LE>,
    iv: [u8; 16],
    secret: [u8; 32],
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
        // let secret_hash = shared_secret.extract::<Sha256>(Some(salt.as_bytes()));

        let mut secret = [0u8; 32];
        // match secret_hash.expand(&[], &mut secret) {
        //     Ok(_) => (),
        //     Err(e) => bail!("Failed to expand shared secret hash: {e}")
        // }

        {
            let mut hasher = Sha256::new();
            hasher.update(salt);
            hasher.update(shared_secret.raw_secret_bytes().as_slice());

            secret.copy_from_slice(&hasher.finalize()[..32]);
        }

        // Initialisation vector is composed of the first 12 bytes of the secret and 0x0000000002
        let mut iv = [0u8; 16];
        iv[..12].copy_from_slice(&secret[..12]);
        iv[12..].copy_from_slice(&[0x00, 0x00, 0x00, 0x02]);

        let cipher = Aes256Ctr64LE::new(&secret.into(), &iv.into());
        Ok((Self {
            iv,
            cipher: Mutex::new(cipher),
            secret,
        }, jwt))
    }

    pub fn decrypt(&self, mut buffer: BytesMut) -> BytesMut {
        assert!(buffer.len() > 9);

        // let mut buffer = Vec::new();
        self.cipher
            .lock()
            .apply_keystream(buffer.as_mut());

        {
            let mut decompressor = DeflateDecoder::new(buffer.as_ref());
            let mut decompressed = Vec::new();
            decompressor.read_to_end(&mut decompressed).unwrap();

            tracing::info!("{:x?}", decompressed);
        }

        tracing::info!("{:x?}", buffer.as_ref());

        let checksum = &buffer.as_ref()[buffer.len() - 8..];
        let computed_checksum = self.compute_checksum(buffer.as_ref());

        tracing::info!("Checksums:\n{:x?}\n{:x?}", checksum, computed_checksum);

        todo!();
    }

    pub fn encrypt(&self, mut buffer: BytesMut) -> BytesMut {
        // self.counter.fetch_add(1, Ordering::SeqCst);
        let checksum = self.compute_checksum(&buffer);

        buffer.put(checksum.as_ref());
        // self.cipher.encrypt();

        todo!("Encryption")
    }

    /// Computes the SHA-256 checksum of the packet.
    fn compute_checksum(&self, data: &[u8]) -> [u8; 8] {
        let mut hasher = Sha256::new();
        hasher.update(0u64.to_le_bytes());
        hasher.update(&data[..data.len() - 8]);
        hasher.update(self.secret);

        let mut checksum = [0u8; 8];
        checksum.copy_from_slice(&hasher.finalize()[..8]);

        checksum
    }
}