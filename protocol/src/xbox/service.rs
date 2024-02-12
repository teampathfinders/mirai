use reqwest::Client;

pub struct XboxService {
    pub(super) http_client: Client
}

impl XboxService {
    pub fn new() -> XboxService {
        XboxService {
            http_client: Client::new()
        }
    }
}