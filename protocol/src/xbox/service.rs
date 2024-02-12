use jsonwebkey as jwk;

use reqwest::Client;
use ring::signature::EcdsaKeyPair;

pub struct XboxService {
    pub(super) http_client: Client,
    pub(super) key_pair: EcdsaKeyPair,
    pub(super) jwk: jwk::JsonWebKey,
    pub(super) rng: ring::rand::SystemRandom
}

impl XboxService {
    pub fn new() -> anyhow::Result<XboxService> {
        let mut jwk = jwk::JsonWebKey::new(jwk::Key::generate_p256());
        jwk.set_algorithm(jwk::Algorithm::ES256)?;

        dbg!(&jwk);

        let pkcs8_private = jwk.key.try_to_der()?;

        let rng = ring::rand::SystemRandom::new();
        let key_pair = ring::signature::EcdsaKeyPair::from_pkcs8(
            &ring::signature::ECDSA_P256_SHA256_FIXED_SIGNING, 
            &pkcs8_private,
            &rng
        )?;

        Ok(XboxService {
            http_client: Client::new(),
            key_pair,
            rng,
            jwk
        })
    }
}