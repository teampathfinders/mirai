use p256::ecdsa::SigningKey;
use rand::rngs::OsRng;
use reqwest::Client;

pub struct XboxService {
    pub(super) http_client: Client,
    pub(super) private_key: SigningKey
}

impl XboxService {
    pub fn new() -> anyhow::Result<XboxService> {
        Ok(XboxService {
            http_client: Client::new(),
            private_key: SigningKey::random(&mut OsRng)
        })
    }
}