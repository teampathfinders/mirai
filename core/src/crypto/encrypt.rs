use std::fmt::{Debug, Formatter};
use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};

use base64::Engine;
use ctr::cipher::KeyIvInit;
use ctr::cipher::StreamCipher;
use jsonwebtoken::Algorithm;
use p384::ecdh::diffie_hellman;
use p384::ecdsa::SigningKey;
use p384::pkcs8::{DecodePublicKey, EncodePrivateKey, EncodePublicKey};
use p384::PublicKey;
use parking_lot::Mutex;
use rand::distributions::Alphanumeric;
use rand::rngs::OsRng;
use rand::Rng;
use sha2::{Digest, Sha256};

use util::bytes::MutableBuffer;
use util::{bail, Result};

type Aes256CtrBE = ctr::Ctr64BE<aes::Aes256>;

/// Use the default Base64 format with no padding.
const BASE64_ENGINE: base64::engine::GeneralPurpose = base64::engine::general_purpose::STANDARD_NO_PAD;

/// Payload of the encryption handshake token
#[derive(serde::Serialize, Debug)]
struct EncryptionTokenClaims<'a> {
    salt: &'a str,
}

/// Used to encrypt and decrypt packets with AES.
pub struct Encryptor {
    /// Cipher used to decrypt packets.
    cipher_decrypt: Mutex<Aes256CtrBE>,
    /// Cipher used to encrypt packets.
    cipher_encrypt: Mutex<Aes256CtrBE>,
    /// Increased by one for every packet sent by the server.
    send_counter: AtomicU64,
    /// Increased by one for every packet received from the client.
    receive_counter: AtomicU64,
    /// Shared secret.
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
        let salt = (0..16).map(|_| OsRng.sample(Alphanumeric) as char).collect::<String>();

        // Generate a random private key for the session.
        let private_key: SigningKey = SigningKey::random(&mut OsRng);
        // Convert the key to the PKCS#8 DER format used by Minecraft.
        let private_key_der = match private_key.to_pkcs8_der() {
            Ok(k) => k,
            Err(e) => bail!(Malformed, "Failed to convert private to PKCS#8 DER format: {e}"),
        };

        // Extract and convert the public key, which will be sent to the client.
        let public_key = private_key.verifying_key();
        let public_key_der = {
            let binary_der = match public_key.to_public_key_der() {
                Ok(d) => d,
                Err(e) => bail!(Malformed, "Failed to convert public key to DER format: {e}"),
            };
            BASE64_ENGINE.encode(binary_der)
        };

        // The typ header is set to none to match the official server software.
        let mut header = jsonwebtoken::Header::new(Algorithm::ES384);
        header.typ = None;
        header.x5u = Some(public_key_der);

        // Create a JWT encoding key using the session's private key.
        let signing_key = jsonwebtoken::EncodingKey::from_ec_der(&private_key_der.to_bytes());
        let claims = EncryptionTokenClaims { salt: &BASE64_ENGINE.encode(&salt) };

        // Generate a JWT containing the server public key and a salt.
        let jwt = jsonwebtoken::encode(&header, &claims, &signing_key)?;
        tracing::debug!("{jwt}");

        let client_public_key = {
            let bytes = BASE64_ENGINE.decode(client_public_key_der)?;
            match PublicKey::from_public_key_der(&bytes) {
                Ok(k) => k,
                Err(e) => bail!(Malformed, "Failed to read DER-encoded client public key: {e}"),
            }
        };

        // Perform the key exchange
        let shared_secret = diffie_hellman(private_key.as_nonzero_scalar(), client_public_key.as_affine());

        // Shared key must be hashed with the salt to produce the shared secret.
        let mut hasher = Sha256::new();
        hasher.update(salt);
        hasher.update(shared_secret.raw_secret_bytes().as_slice());

        let mut secret = [0u8; 32];
        secret.copy_from_slice(&hasher.finalize()[..32]);

        tracing::debug!("{secret:?}");

        // Initialisation vector is composed of the first 12 bytes of the secret and 0x0000000002
        let mut iv = [0u8; 16];
        iv[..12].copy_from_slice(&secret[..12]);
        iv[12..].copy_from_slice(&[0x00, 0x00, 0x00, 0x02]);

        let cipher = Aes256CtrBE::new(&secret.into(), &iv.into());
        Ok((
            Self {
                send_counter: AtomicU64::new(0),
                receive_counter: AtomicU64::new(0),
                cipher_decrypt: Mutex::new(cipher.clone()),
                cipher_encrypt: Mutex::new(cipher),
                secret,
            },
            jwt,
        ))
    }

    /// Decrypts a packet and verifies its checksum.
    ///
    /// If the checksum does not match, a [`BadPacket`](util::ErrorKind::Malformed) error is returned.
    /// The client must be disconnected if this fails, because the data has probably been tampered with.
    pub fn decrypt(&self, buffer: &mut MutableBuffer) -> anyhow::Result<()> {
        if buffer.len() < 9 {
            bail!(Malformed, "Encrypted buffer must be at least 9 bytes, received {}", buffer.len());
        }

        self.cipher_decrypt.lock().apply_keystream(buffer.as_mut_slice());
        let counter = self.receive_counter.fetch_add(1, Ordering::SeqCst);

        let slice = buffer.as_slice();
        let checksum = &slice[slice.len() - 8..];
        let computed_checksum = self.compute_checksum(&slice[..slice.len() - 8], counter);

        if !checksum.eq(&computed_checksum) {
            bail!(Malformed, "Encryption checksums do not match");
        }

        // Remove checksum from data.
        buffer.truncate(buffer.len() - 8);

        Ok(())
    }

    /// Encrypts a packet and appends the computed checksum.
    pub fn encrypt(&self, buffer: &mut MutableBuffer) -> anyhow::Result<()> {
        let counter = self.send_counter.fetch_add(1, Ordering::SeqCst);
        // Exclude 0xfe header from checksum calculations.
        let checksum = self.compute_checksum(&buffer.as_ref()[1..], counter);
        buffer.write_all(&checksum)?;

        self.cipher_encrypt.lock().apply_keystream(&mut buffer.as_mut_slice()[1..]);

        Ok(())
    }

    /// Computes the SHA-256 checksum of the packet.
    ///
    /// This checksum can be used to verify that the packet has not been modified.
    /// It consists of 8 bytes and is appended to the encrypted payload.
    fn compute_checksum(&self, data: &[u8], counter: u64) -> [u8; 8] {
        let mut hasher = Sha256::new();
        hasher.update(counter.to_le_bytes());
        hasher.update(data);
        hasher.update(self.secret);

        // Minecraft uses only the first 8 bytes of the hash.
        let mut checksum = [0u8; 8];
        checksum.copy_from_slice(&hasher.finalize()[..8]);

        checksum
    }
}
