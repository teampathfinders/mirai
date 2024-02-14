//! Used to obtain actual Xbox tokens

use std::{io::Write, time::SystemTime};

use base64::Engine;
use ecdsa::{elliptic_curve::group::prime::PrimeCurveAffine, signature::{DigestSigner, Signer}};
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};
use util::{PVec, BinaryWrite};
use uuid::Uuid;
use p256::ecdsa::{Signature, SigningKey, VerifyingKey};

use super::{LiveToken, XboxService};

#[derive(Debug)]
pub struct XboxToken {

}   

#[derive(Debug)]
struct DeviceToken {

}

impl XboxService {
    pub async fn fetch_xbox_token(&self, live_token: &LiveToken) -> anyhow::Result<XboxToken> {
        let device_token = self.fetch_device_token().await?;

        todo!()
    }

    async fn fetch_device_token(&self) -> anyhow::Result<DeviceToken> {
        const ENGINE: base64::engine::GeneralPurpose = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        
        let public_key = self.private_key.verifying_key();

        let curve_point = public_key.to_encoded_point(false);
        let (x, y) = (curve_point.x().unwrap(), curve_point.y().unwrap());

        let b64_x = ENGINE.encode(x.as_slice());
        let b64_y = ENGINE.encode(y.as_slice());

        let random_uuid = Uuid::new_v4();
        // let key_x = ENGINE.encode(key_pair.)

        let request_content = serde_json::json!({
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT",
            "Properties": {
                "AuthMethod": "ProofOfPossession",
                "Id": format!("{{{random_uuid}}}"),
                "DeviceType": "Android",
                "Version": "10",
                "ProofKey": {
                    "crv": "P-256",
                    "alg": "ES256",
                    "use": "sig",
                    "kty": "EC",
                    "x": b64_x,
                    "y": b64_y
                }
            }
        });

        let request_payload = serde_json::to_string(&request_content)?;
        let signature = self.sign("device/authenticate", &request_payload, None)?;

        println!("{request_payload}");

        let request = self.http_client
            .post("https://device.auth.xboxlive.com/device/authenticate")
            .header("x-xbl-contract-version", "1")
            // .header("Content-Type", "application/json")
            // .header("Accept", "application/json")
            .header("Signature", signature)
            .body(request_payload)
            .build()?;

        let response = self.http_client
            .execute(request)
            .await?;

        dbg!(&response);
        dbg!(response.text().await?);

        todo!()
    }

    fn sign(&self, url: &str, payload: &str, auth_token: Option<&str>) -> anyhow::Result<String> {
        const ENGINE: base64::engine::GeneralPurpose = base64::engine::general_purpose::STANDARD;

        let timestamp = {
            let elapsed = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
            ((elapsed.as_millis() / 1000) + 11_644_473_600) * 1_0000_000
        } as u64;

        // let mut writer = BVec::alloc();

        // writer.write_u32_be(1)?;
        // writer.write_u8(0)?;

        // writer.write_u64_be(timestamp)?;
        // writer.write_u8(0)?;

        // writer.write_all("POST".as_bytes())?;
        // writer.write_u8(0)?;

        // writer.write_all(url.as_bytes())?;
        // writer.write_u8(0)?;

        // writer.write_all(auth_token.unwrap_or_default().as_bytes())?;
        // writer.write_u8(0)?;

        // writer.write_all(payload.as_bytes())?;
        // writer.write_u8(0)?;

        let mut hasher = Sha256::new();

        let mut buf = Vec::new();

        buf.write_all(&[0, 0, 0, 1, 0])?;
        buf.write_u64_be(timestamp)?;
        buf.write_u8(0)?;

        hasher.update(buf.as_slice());

        hasher.update("POST");
        hasher.update([0]);

        hasher.update(url);
        hasher.update([0]);

        hasher.update(auth_token.unwrap_or_default());
        hasher.update([0]);

        hasher.update(payload);
        hasher.update([0]);

        // let hash = hasher.finalize();
        let signed: Signature = self.private_key.sign_digest(hasher);

        let r = signed.r();
        let s = signed.s();

        let rb = r.to_bytes();
        let sb = s.to_bytes();

        let mut signature = Vec::with_capacity(12);
        signature.write_all(&[0, 0, 0, 1])?;
        signature.write_u64_be(timestamp)?;
        signature.write_all(rb.as_slice())?;
        signature.write_all(sb.as_slice())?;



        Ok(ENGINE.encode(&signature))
    }
}

// use std::{borrow::Cow, io::Write, time::{Instant, SystemTime}};
// use base64::Engine;
// use reqwest::StatusCode;
// use sha2::Digest;

// use jsonwebkey as jwk;

// use util::{BVec, BinaryWrite, Reusable};
// use uuid::Uuid;

// use super::XboxService;

// const ENGINE: base64::engine::GeneralPurpose = base64::engine::general_purpose::STANDARD_NO_PAD;

// const XBOX_RELYING_PARTY: &'static str = "http://auth.xboxlive.com";
// const XBOX_DEVICE_AUTHENTICATE_URL: &'static str = "https://device.auth.xboxlive.com/device/authenticate";

// pub struct XboxToken {

// }

// #[derive(serde::Serialize)]
// struct ProofKey<'a> {
//     kty: &'static str,
//     x: &'a str,
//     y: &'a str,
//     crv: &'static str,
//     alg: &'static str,
//     #[serde(rename = "use")]
//     key_use: &'static str
// }

// #[derive(serde::Serialize)]
// #[serde(rename_all = "PascalCase")]
// struct DeviceProperties<'a> {
//     auth_method: &'static str,
//     id: &'a str,
//     device_type: &'static str,
//     version: &'static str,
//     proof_key: ProofKey<'a>,
//     serial_number: &'a str
// }

// #[derive(serde::Serialize)]
// #[serde(rename_all = "PascalCase")]
// struct UserProperties<'a> {
//     auth_method: &'static str,
//     site_name: &'static str,
//     rps_ticket: &'a str
// }

// #[derive(serde::Serialize)]
// #[serde(untagged)]
// enum TokenRequestProperties<'a> {
//     User(UserProperties<'a>),
//     Device(DeviceProperties<'a>),
//     Title(),
//     Xsts()
// }

// #[derive(serde::Serialize)]
// #[serde(rename_all = "PascalCase")]
// struct TokenRequest<'a> {
//     relying_party: &'static str,
//     token_type: &'static str,
//     properties: TokenRequestProperties<'a>
// }

// #[derive(serde::Serialize)]
// #[serde(rename_all = "PascalCase")]
// struct UserTokenRequest<'a> {
//     relying_party: &'static str,
//     token_type: &'static str,
//     properties: TokenRequestProperties<'a>
// }

// #[derive(serde::Deserialize)]
// #[serde(rename_all = "PascalCase")]
// struct UserToken {
//     issue_instant: String,
//     not_after: String,
//     token: String
// }

// impl XboxService {
//     pub async fn fetch_minecraft_token(&self) -> anyhow::Result<XboxToken> {
//         let xsts = self.fetch_xsts_token().await?;
//         // tracing::debug!("xsts: {xsts}");

//         todo!()
//     }

//     async fn fetch_xsts_token(&self) -> anyhow::Result<()> {
//         let live_token = self.fetch_live_token().await?;
//         let user_token = self.fetch_user_token(&live_token.access_token).await?;

//         Ok(())
//     }

//     async fn fetch_user_token(&self, access_token: &str) -> anyhow::Result<UserToken> {
//         let rps_ticket = format!("t={access_token}");
//         let request_content = TokenRequest {
//             relying_party: "http://auth.xboxlive.com",
//             token_type: "JWT",
//             properties: TokenRequestProperties::User(UserProperties {
//                 auth_method: "RPS",
//                 site_name: "user.auth.xboxlive.com",
//                 rps_ticket: &rps_ticket
//             })
//         };

//         let request_json = serde_json::to_string(&request_content)?;
//         let signature = self.sign_request_payload(
//             "https://user.auth.xboxlive.com/user/authenticate", &request_json
//         )?;

//         let request = self.http_client
//             .post("https://user.auth.xboxlive.com/user/authenticate")
//             .header("Content-Type", "application/json")
//             .header("accept", "application/json")
//             .header("x-xbl-contract-version", "2")
//             .header("Cache-Control", "no-store, must-revalidate, no-cache")
//             .header("Signature", signature)
//             .body(request_json)
//             .build()?;

//         let response = self.http_client.execute(request).await?;
//         if response.status() != StatusCode::OK {
//             anyhow::bail!("User token request failed: status code {}", response.status());
//         }

//         let body_json = response.text().await?;
//         let body: UserToken = serde_json::from_str(&body_json)?;
        
//         Ok(body)
//     }

//     pub async fn fetch_xbox_token(&self) -> anyhow::Result<XboxToken> {
//         // let proof_key = self.jwk.key
//         //     .to_public()
//         //     .ok_or_else(|| anyhow::anyhow!("Failed to generate public JSON web key"))?
//         //     .try_to_der()?;

//         // let mut base64 = BVec::alloc_with_capacity(proof_key.len() * 4 / 3 + 4);
        
//         // // Set length to capacity. The base64 engine will otherwise complain about the slice being too small..
//         // let cap = base64.capacity();
//         // base64.resize(cap, 0);

//         // let written = ENGINE.encode_slice(proof_key.as_slice(), &mut base64)?;
//         // base64.truncate(written);

//         // let base64_string = String::from_utf8(base64.into_inner())?;

//         let (x, y) = match self.jwk.key.as_ref() {
//             jwk::Key::EC {
//                 curve: jwk::Curve::P256 {
//                     d, x, y
//                 }
//             } => {
//                 (x, y)
//             },
//             _ => unreachable!()
//         };

//         let id_uuid = XboxService::next_uuid();
//         let serial_uuid = XboxService::next_uuid();

//         let base64_x = ENGINE.encode(x);
//         let base64_y = ENGINE.encode(y);

//         let proof_request = TokenRequest {
//             relying_party: XBOX_RELYING_PARTY,
//             token_type: "JWT",
//             properties: TokenRequestProperties::Device(DeviceProperties {
//                 auth_method: "ProofOfPossession",
//                 id: &id_uuid,
//                 device_type: "Android",
//                 version: "10",
//                 serial_number: &serial_uuid,
//                 proof_key: ProofKey {
//                     x: &base64_x, y: &base64_y,
//                     kty: "EC",
//                     crv: "P-256",
//                     alg: "ES256",
//                     key_use: "sig"
//                 }
//             })
//         };

//         let request_content = serde_json::to_string(&proof_request)?;
//         tracing::debug!("{request_content}");

//         let signed_payload = self.sign_request_payload(XBOX_DEVICE_AUTHENTICATE_URL, &request_content)?;
//         tracing::debug!("{signed_payload}");

//         let request = self.http_client
//             .post(XBOX_DEVICE_AUTHENTICATE_URL)
//             .header("Content-Type", "application/json")
//             .header("Signature", &signed_payload)
//             .header("Cache-Control", "no-store, must-revalidate, no-cache")
//             .header("x-xbl-contract-version", 1)
//             .body(request_content)
//             .build()?;

//         let response = self.http_client.execute(request).await?;
//         // let _reuse = Reusable::from(base64_string.into_bytes());

//         // let body_json = response.text().await?;

//         tracing::info!("{response:?}");

//         // Return string contents to memory pool
//         let _bvec = Reusable::from(signed_payload.into_bytes());

//         todo!()
//     }

//     #[inline]
//     fn next_uuid() -> String {
//         format!("{{{}}}", Uuid::new_v4())
//         // Uuid::new_v3(
//         //     &uuid::uuid!("6ba7b811-9dad-11d1-80b4-00c04fd430c8"), 

//         // )
//     }

//     fn sign_request_payload(&self, uri: &str, payload: &str) -> anyhow::Result<String> {
//         let timestamp = {
//             let elapsed = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
//             ((elapsed.as_millis() / 1000) + 11_644_473_600) * 1_0000_000
//         } as u64;

//         let size_hint = 5 + 9 + 5 + uri.len() + 1 + 0;
//         let mut writer = BVec::alloc_with_capacity(size_hint);

//         writer.write_i32_be(1)?; // Policy version
//         writer.write_u8(0)?;
//         writer.write_u64_be(timestamp)?;
//         writer.write_u8(0)?;
//         writer.write_all("POST".as_bytes())?;
//         writer.write_u8(0)?;
//         writer.write_all(uri.as_bytes())?;
//         writer.write_u8(0)?;
//         // Empty field
//         writer.write_u8(0)?;
//         writer.write_all(payload.as_bytes())?;
//         writer.write_u8(0)?;

//         let signature = self.key_pair.sign(&self.rng, payload.as_bytes())?;

//         let mut sig_writer = BVec::alloc_with_capacity(signature.as_ref().len() + 12);
//         sig_writer.write_u8(1)?; // Policy version
//         sig_writer.write_u64_be(timestamp)?;
//         sig_writer.write_all(signature.as_ref())?;

//         let mut base64 = BVec::alloc_with_capacity(sig_writer.len() * 4 / 3 + 4);

//         // Set length to capacity. The base64 engine will otherwise complain about the slice being too small..
//         let cap = base64.capacity();
//         base64.resize(cap, 0);

//         let written = ENGINE.encode_slice(sig_writer.as_ref(), &mut base64)?;
//         base64.truncate(written);

//         let string = String::from_utf8(base64.into_inner())?;
//         Ok(string)
//     }
// }