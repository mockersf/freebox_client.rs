use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::free_client::Response;

#[derive(Deserialize, Debug)]
struct ApiVersion {
    uid: String,
    device_name: String,
    api_version: String,
    api_base_url: String,
    device_type: String,
    api_domain: String,
    https_available: bool,
    https_port: u16,
}

#[derive(Serialize, Debug)]
struct Auth {
    app_id: String,
    app_name: String,
    app_version: String,
    device_name: String,
}

#[derive(Serialize, Debug)]
struct StartSession {
    app_id: String,
    password: String,
}

#[derive(Deserialize, Debug)]
struct Authorize {
    app_token: String,
    track_id: u8,
}
#[derive(Deserialize, Debug)]
struct AuthorizeTrack {
    status: String,
}
#[derive(Deserialize, Debug)]
struct Login {
    logged_in: bool,
    challenge: String,
}
#[derive(Deserialize, Debug)]
struct Session {
    session_token: String,
}

#[derive(Error, Debug)]
#[error("invalid version error")]
struct InvalidVersionError;

pub struct UnauthorizedClient<'a> {
    http_client: &'a reqwest::Client,
    app_id: String,
}

impl<'a> UnauthorizedClient<'a> {
    pub fn new(http_client: &'a reqwest::Client, app_id: &str) -> UnauthorizedClient<'a> {
        UnauthorizedClient {
            http_client,
            app_id: String::from(app_id),
        }
    }

    pub async fn authorize(&self, base_url: &str) -> anyhow::Result<String> {
        let res = self
            .http_client
            .post(&format!("{}{}", base_url, "login/authorize"))
            .json(&Auth {
                app_id: self.app_id.clone(),
                app_name: self.app_id.clone(),
                app_version: String::from("1.0"),
                device_name: self.app_id.clone(),
            })
            .send()
            .await?
            .json::<Response<Authorize>>()
            .await?;

        loop {
            let status = self
                .http_client
                .get(&format!(
                    "{}{}/{}",
                    base_url, "login/authorize", res.result.track_id
                ))
                .send()
                .await?
                .json::<Response<AuthorizeTrack>>()
                .await?
                .result
                .status;
            if status == "granted" {
                break;
            }
        }

        Ok(res.result.app_token)
    }

    async fn get_api_version(&self) -> anyhow::Result<ApiVersion> {
        Ok(self
            .http_client
            .get("http://mafreebox.freebox.fr/api_version")
            .send()
            .await?
            .json::<ApiVersion>()
            .await?)
    }

    pub async fn get_api_domain(&self) -> anyhow::Result<String> {
        Ok(self.get_api_version().await?.api_domain)
    }

    pub async fn get_base_url(&self) -> anyhow::Result<String> {
        let api_version = self.get_api_version().await?;

        Ok(format!(
            "https://{}:{}{}v{}/",
            api_version.api_domain,
            api_version.https_port,
            api_version.api_base_url,
            api_version
                .api_version
                .split('.')
                .next()
                .ok_or(InvalidVersionError)?
        ))
    }

    pub async fn get_session(&self, base_url: &str, app_token: &str) -> anyhow::Result<String> {
        let challenge = self
            .http_client
            .get(&format!("{}{}", base_url, "login/"))
            .send()
            .await?
            .json::<Response<Login>>()
            .await?
            .result
            .challenge;

        let coded = hmacsha1::hmac_sha1(app_token.as_bytes(), challenge.as_bytes());
        let password = coded.iter().map(|b| format!("{:02x}", b)).collect();

        Ok(self
            .http_client
            .post(&format!("{}{}", base_url, "login/session"))
            .json(&StartSession {
                app_id: self.app_id.clone(),
                password,
            })
            .send()
            .await?
            .json::<Response<Session>>()
            .await?
            .result
            .session_token)
    }
}
