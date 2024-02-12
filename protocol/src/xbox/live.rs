//! Used to obtain live tokens from login.live.com.

use std::time::{Duration, Instant};

use reqwest::StatusCode;

use super::XboxService;

const LIVE_DEVICE_CODE_URL: &'static str = "https://login.live.com/oauth20_connect.srf";
const LIVE_POLL_URL: &'static str = "https://login.live.com/oauth20_token.srf";
const LIVE_AUTH_SCOPE: &'static str = "service::user.auth.xboxlive.com::MBI_SSL";
const LIVE_AUTH_CLIENT_ID: &'static str = "0000000048183522";
const LIVE_AUTH_GRANT_TYPE: &'static str = "urn:ietf:params:oauth:grant-type:device_code";

#[derive(serde::Deserialize, Debug)]
struct LiveDeviceCodeResponse {
    user_code: String,
    device_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: u64
}

#[derive(serde::Deserialize, Debug)]
struct LivePollResponse {
    token_type: String,
    expires_in: u64,
    scope: String,
    access_token: String,
    user_id: String,
    refresh_token: String
}

pub struct LiveToken {
    pub expires_at: Instant,
    pub access_token: String,
    pub refresh_token: String
}

impl LiveToken {
    pub fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }
}

impl XboxService {
    /// Requests a live token from Microsoft.
    pub async fn request_live_token(&self) -> anyhow::Result<LiveToken> {
        let request = self.http_client
            .post(LIVE_DEVICE_CODE_URL)
            .form(&[
                ("scope", LIVE_AUTH_SCOPE),
                ("client_id", LIVE_AUTH_CLIENT_ID),
                ("response_type", "device_code")
            ])
            .build()?;

        let response = self.http_client.execute(request).await?;
        if response.status() != StatusCode::OK {
            anyhow::bail!("Failed to request device code token");
        }

        let body_json = response.text().await?;
        let body: LiveDeviceCodeResponse = serde_json::from_str(&body_json)?;

        tracing::info!("Enter link code {} in the form that just opened and log in with your account", body.user_code);
        open::that_in_background(body.verification_uri);

        let poll_response = match tokio::time::timeout(
            Duration::from_secs(body.expires_in), 
            self.poll_device_token(body.interval, &body.device_code)
        ).await {
            Ok(res) => res?,
            Err(_) => anyhow::bail!("The device code login token has expired")
        };

        tracing::info!("Succesfully logged into Microsoft services");

        Ok(LiveToken {
            expires_at: Instant::now() + Duration::from_secs(poll_response.expires_in),
            access_token: poll_response.access_token,
            refresh_token: poll_response.refresh_token
        })
    }

    /// Polls the Microsoft endpoint every `interval` seconds to check whether the user has logged in.
    async fn poll_device_token(&self, interval: u64, device_code: &str) -> anyhow::Result<LivePollResponse> {
        let mut interval = tokio::time::interval(Duration::from_secs(interval));
        let response = loop {
            let poll_request = self.http_client
                .post(LIVE_POLL_URL)
                .form(&[
                    ("client_id", LIVE_AUTH_CLIENT_ID),
                    ("grant_type", LIVE_AUTH_GRANT_TYPE),
                    ("device_code", device_code)
                ])
                .build()?;

            let response = self.http_client.execute(poll_request).await?;
            match response.status() {
                StatusCode::OK => {
                    let body_json = response.text().await?;
                    break serde_json::from_str(&body_json)?
                },
                StatusCode::BAD_REQUEST => {
                    // User has not logged in yet, continue polling.
                },
                code => {
                    anyhow::bail!("Polling live token failed: status code {code}")
                }          
            }

            interval.tick().await;
        };

        Ok(response)
    }
}