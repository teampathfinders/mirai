//! Used to obtain live tokens from login.live.com.

use std::time::{Duration, Instant};

use reqwest::StatusCode;

use super::XboxService;

const LIVE_CLIENT_ID: &str = "0000000048183522";

#[derive(Debug)]
pub struct LiveToken {
    user_id: String,
    expiry: Instant,
    pub(super) access_token: String,
    refresh_token: String
}

impl LiveToken {
    /// Whether the live token is still valid.
    pub fn is_valid(&self) -> bool {
        Instant::now() > self.expiry
    }
}

#[derive(serde::Deserialize, Debug)]
struct LiveResponse {
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

impl XboxService {
    /// Requests a live token from Microsoft.
    pub async fn fetch_live_token(&self) -> anyhow::Result<LiveToken> {
        let request = self.http_client
            .post("https://login.live.com/oauth20_connect.srf")
            .form(&[
                ("scope", "service::user.auth.xboxlive.com::MBI_SSL"),
                ("client_id", LIVE_CLIENT_ID),
                ("response_type", "device_code")
            ])
            .build()?;

        let response = self.http_client.execute(request).await?;
        if response.status() != StatusCode::OK {
            anyhow::bail!("Failed to request device code token");
        }

        let body_json = response.text().await?;
        let body: LiveResponse = serde_json::from_str(&body_json)?;

        // tracing::info!("Enter link code {} in the form that just opened and log in with your account", body.user_code);
        open::that_in_background(format!("https://microsoft.com/link?otc={}", body.user_code));

        let poll_response = match tokio::time::timeout(
            Duration::from_secs(body.expires_in), 
            self.poll_device_token(body.interval, &body.device_code)
        ).await {
            Ok(res) => res?,
            Err(_) => anyhow::bail!("The device code login token has expired")
        };

        tracing::info!("Succesfully logged into Microsoft services");

        Ok(LiveToken {
            user_id: poll_response.user_id,
            expiry: Instant::now() + Duration::from_secs(poll_response.expires_in),
            access_token: poll_response.access_token,
            refresh_token: poll_response.refresh_token
        })
    }

    /// Polls the Microsoft endpoint every `interval` seconds to check whether the user has logged in.
    async fn poll_device_token(&self, interval: u64, device_code: &str) -> anyhow::Result<LivePollResponse> {
        let mut interval = tokio::time::interval(Duration::from_secs(interval));
        let response = loop {
            let poll_request = self.http_client
                .post("https://login.live.com/oauth20_token.srf")
                .form(&[
                    ("client_id", LIVE_CLIENT_ID),
                    ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                    ("device_code", device_code)
                ])
                .build()?;

            let response = self.http_client.execute(poll_request).await?;
            match response.status() {
                StatusCode::OK => {
                    break response.json().await?;
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

// use std::{borrow::Cow, time::{Duration, Instant}};

// use rand::Rng;
// use reqwest::{Client, StatusCode};
// use std::sync::mpsc;

// use super::XboxService;

// const LIVE_DEVICE_CODE_URL: &'static str = "https://login.live.com/oauth20_connect.srf";
// const LIVE_POLL_URL: &'static str = "https://login.live.com/oauth20_token.srf";
// const LIVE_AUTH_SCOPE: &'static str = "service::user.auth.xboxlive.com::MBI_SSL";
// // const LIVE_AUTH_CLIENT_ID: &'static str = "0000000048183522";
// const LIVE_AUTH_CLIENT_ID: &'static str = "00000000441cc96b";
// const LIVE_AUTH_GRANT_TYPE: &'static str = "urn:ietf:params:oauth:grant-type:device_code";

// #[derive(serde::Deserialize, Debug)]
// struct LiveDeviceCodeResponse {
//     user_code: String,
//     device_code: String,
//     verification_uri: String,
//     expires_in: u64,
//     interval: u64
// }

// #[derive(serde::Deserialize, Debug)]
// struct LivePollResponse {
//     token_type: String,
//     expires_in: u64,
//     scope: String,
//     access_token: String,
//     user_id: String,
//     refresh_token: String
// }

// pub struct LiveToken {
//     pub user_id: String,
//     pub expires_at: Instant,
//     pub access_token: String,
//     pub refresh_token: String
// }

// impl LiveToken {
//     pub fn is_expired(&self) -> bool {
//         Instant::now() > self.expires_at
//     }
// }

// #[derive(serde::Deserialize, Debug)]
// struct Query {
//     code: String,
//     state: String
// }

// #[derive(serde::Deserialize, Debug)]
// struct AccessToken {
//     access_token: String
// }

// #[derive(serde::Deserialize, Debug)]
// struct Xui {
//     #[serde(rename = "uhs")]
//     user_hash: String
// }

// #[derive(serde::Deserialize, Debug)]
// struct DisplayClaims {
//     xui: (Xui, )
// }

// #[derive(serde::Deserialize, Debug)]
// #[serde(rename_all = "PascalCase")]
// struct UserToken {
//     issue_instant: String,
//     not_after: String,
//     token: String,
//     display_claims: DisplayClaims
// }

// #[derive(serde::Deserialize, Debug)]
// struct Item {
//     pub name: Cow<'static, str>
// }

// #[derive(serde::Deserialize, Debug)]
// #[serde(rename_all = "PascalCase")]
// struct Store {
//     items: Vec<Item>,
//     #[serde(rename = "keyId")]
//     key_id: String
// }

// impl XboxService {
//     pub async fn fetch_live_token(&self) -> anyhow::Result<LiveToken> {
//         let client_id = std::env::var("CLIENT_ID")?;
//         let client_secret = std::env::var("CLIENT_SECRET")?;
//         let redirect_uri = std::env::var("REDIRECT_URI")?;

//         let state = rand::thread_rng().sample_iter(rand::distributions::Alphanumeric).take(16).map(char::from).collect::<String>();
//         let url = format!(
//             "\
//                 https://login.live.com/oauth20_authorize.srf\
//                 ?client_id={client_id}\
//                 &response_type=code\
//                 &redirect_uri={redirect_uri}\
//                 &scope=XboxLive.signin%20offline_access\
//                 &state={state}\
//             "
//         );

//         open::that_detached(&url)?;

//         let query = self.poll_authorize().await?;
//         tracing::debug!("Obtained authorization code");

//         let client = Client::new();

//         let access_token: AccessToken = client
//             .post("https://login.live.com/oauth20_token.srf")
//             .form(&[
//                 ("client_id", client_id),
//                 ("client_secret", client_secret),
//                 ("code", query.code),
//                 ("grant_type", "authorization_code".to_owned()),
//                 ("redirect_uri", redirect_uri)
//             ])
//             .send().await?.json().await?;
        
//         tracing::debug!("Obtained RPS token");

//         let access_token = access_token.access_token;
//         let user_request = serde_json::json!({
//             "Properties": {
//                 "AuthMethod": "RPS",
//                 "SiteName": "user.auth.xboxlive.com",
//                 "RpsTicket": format!("d={access_token}")
//             },
//             "RelyingParty": "http://auth.xboxlive.com",
//             "TokenType": "JWT"
//         });

//         let user_response: UserToken = client
//             .post("https://user.auth.xboxlive.com/user/authenticate")
//             .json(&user_request)
//             .send().await?.json().await?;

//         tracing::debug!("Obtained user token");

//         let token = user_response.token;
//         let user_hash = user_response.display_claims.xui.0.user_hash;

//         let xsts_request = serde_json::json!({
//             "Properties": {
//                 "SandboxId": "RETAIL",
//                 "UserTokens": [token],
//             },
//             "RelyingParty": "rp://api.minecraftservices.com/",
//             "TokenType": "JWT"
//         });

//         let xsts_response: UserToken = client
//             .post("https://xsts.auth.xboxlive.com/xsts/authorize")
//             .json(&xsts_request)
//             .send().await?.json().await?;

//         let token = xsts_response.token;

//         tracing::debug!("Obtained XSTS token");
//         let access_request = serde_json::json!({
//             "identityToken": format!("XBL3.0 x={};{}", user_hash, token)
//         });

//         let access_token = client
//             .post("https://api.minecraftservices.com/authentication/login_with_xbox")
//             .json(&access_request)
//             .send().await?;

//         dbg!(access_token.text().await?);

//         // tracing::debug!("Authenticated with Minecraft");
//         // let access_token = access_token.access_token;

//         // let store: Store = client 
//         //     .get("https://api.minecraftservices.com/entitlements/mcstore")
//         //     .bearer_auth(&access_token)
//         //     .send().await?.json().await?;

//         // dbg!(&store);

//         todo!()
//     }

//     async fn poll_authorize(&self) -> anyhow::Result<Query> {
//         use warp::Filter;

//         let (sender, mut receiver) = mpsc::sync_channel(0);

//         let route = warp::path("auth")
//             .and(warp::filters::query::query())
//             .map(move |query: Query| {
//                 sender.send(query).unwrap();
//                 "Authorization granted, you can close this window"
//             });

//         tokio::spawn(warp::serve(route).run(([127, 0, 0, 1], 8080)));

//         let query = receiver.recv()?;
//         Ok(query)
//     }

//     // /// Requests a live token from Microsoft.
//     // pub async fn fetch_live_token(&self) -> anyhow::Result<LiveToken> {
//     //     let request = self.http_client
//     //         .post(LIVE_DEVICE_CODE_URL)
//     //         .form(&[
//     //             ("scope", LIVE_AUTH_SCOPE),
//     //             ("client_id", LIVE_AUTH_CLIENT_ID),
//     //             ("response_type", "device_code")
//     //         ])
//     //         .build()?;

//     //     let response = self.http_client.execute(request).await?;
//     //     if response.status() != StatusCode::OK {
//     //         anyhow::bail!("Failed to request device code token");
//     //     }

//     //     let body_json = response.text().await?;
//     //     let body: LiveDeviceCodeResponse = serde_json::from_str(&body_json)?;

//     //     // tracing::info!("Enter link code {} in the form that just opened and log in with your account", body.user_code);
//     //     open::that_in_background(format!("https://microsoft.com/link?otc={}", body.user_code));

//     //     let poll_response = match tokio::time::timeout(
//     //         Duration::from_secs(body.expires_in), 
//     //         self.poll_device_token(body.interval, &body.device_code)
//     //     ).await {
//     //         Ok(res) => res?,
//     //         Err(_) => anyhow::bail!("The device code login token has expired")
//     //     };

//     //     tracing::info!("Succesfully logged into Microsoft services");

//     //     Ok(LiveToken {
//     //         user_id: poll_response.user_id,
//     //         expires_at: Instant::now() + Duration::from_secs(poll_response.expires_in),
//     //         access_token: poll_response.access_token,
//     //         refresh_token: poll_response.refresh_token
//     //     })
//     // }

//     // /// Polls the Microsoft endpoint every `interval` seconds to check whether the user has logged in.
//     // async fn poll_device_token(&self, interval: u64, device_code: &str) -> anyhow::Result<LivePollResponse> {
//     //     let mut interval = tokio::time::interval(Duration::from_secs(interval));
//     //     let response = loop {
//     //         let poll_request = self.http_client
//     //             .post(LIVE_POLL_URL)
//     //             .form(&[
//     //                 ("client_id", "SECRET"),
//     //                 ("client_secret", "SECRET"),
//     //                 ("code", )
//     //                 ("grant_type", "authorization_code"),
//     //                 ("redirect_uri", "http://localhost")
//     //             ])
//     //             .build()?;

//     //         let response = self.http_client.execute(poll_request).await?;
//     //         match response.status() {
//     //             StatusCode::OK => {
//     //                 break response.json().await?;
//     //             },
//     //             StatusCode::BAD_REQUEST => {
//     //                 // User has not logged in yet, continue polling.
//     //             },
//     //             code => {
//     //                 anyhow::bail!("Polling live token failed: status code {code}")
//     //             }          
//     //         }

//     //         interval.tick().await;
//     //     };

//     //     Ok(response)
//     // }
// }