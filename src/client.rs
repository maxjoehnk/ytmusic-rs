use std::convert::TryInto;
use std::time::{SystemTime, UNIX_EPOCH};
use headers::Header;
use regex::Regex;

use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use sha1::{Sha1, Digest};
use surf::{Body, Client, Config, Url};
use crate::YoutubeMusicError;

const API_KEY: &str = "AIzaSyC9XL3ZjWddXya6X74dJoCTL-WEYFDNX30";
const YTM_DOMAIN: &str = "https://music.youtube.com";
const URL: &str = "https://music.youtube.com/youtubei/v1/";

const SAPISID_COOKIE_NAME: &str = "__Secure-3PAPISID";

#[derive(Debug, Clone)]
pub struct YoutubeMusicClient {
    user_id: Option<String>,
    client: Client,
    sapisid: String,
    visitor_id: Option<String>,
}

fn build_client(cookies: &str) -> Result<Client, surf::Error> {
    let client: Client = Config::new()
        .set_base_url(Url::parse(URL)?)
        .add_header("x-origin", YTM_DOMAIN)?
        .add_header("origin", YTM_DOMAIN)?
        .add_header("referer", YTM_DOMAIN)?
        .add_header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:72.0) Gecko/20100101 Firefox/72.0")?
        .add_header("accept", "*/*")?
        .add_header("content-type", "application/json")?
        .add_header("content-encoding", "gzip")?
        .add_header("cookie", cookies)?
        .try_into()?;

    Ok(client)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
struct YoutubeConfig {
    visitor_data: String
}

fn get_sapisid_from_cookies(cookies: &str) -> Result<String, YoutubeMusicError> {
    let header_value = http::header::HeaderValue::from_str(cookies)
        .map_err(|err| YoutubeMusicError::InvalidCookieError(err.to_string()))?;
    let cookies = headers::Cookie::decode(&mut [header_value].iter())
        .map_err(|err| YoutubeMusicError::InvalidCookieError(err.to_string()))?;

    let sapisid = cookies.get(SAPISID_COOKIE_NAME)
        .ok_or(YoutubeMusicError::MissingSapisidCookieError)?;

    Ok(sapisid.to_string())
}

impl YoutubeMusicClient {
    pub fn new(cookies: &str, user_id: Option<String>) -> Result<Self, YoutubeMusicError> {
        let client = build_client(cookies).unwrap();
        let sapisid = get_sapisid_from_cookies(cookies)?;

        Ok(YoutubeMusicClient {
            user_id,
            client,
            sapisid,
            visitor_id: None,
        })
    }

    pub fn set_visitor_id(&mut self, visitor_id: String) {
        self.visitor_id = Some(visitor_id);
    }

    pub async fn fetch_visitor_id(&mut self) -> Result<(), YoutubeMusicError> {
        let regex = Regex::new(r"ytcfg\.set\s*\(\s*(\{.+?\})\s*\)\s*;").unwrap();
        let res = self.client.get(YTM_DOMAIN).await?.body_string().await?;
        let matches = regex.captures(&res).ok_or(YoutubeMusicError::MissingVisitorId)?;
        let config: YoutubeConfig = serde_json::from_str(&matches[1])?;

        self.visitor_id = Some(config.visitor_data);

        Ok(())
    }

    pub(crate) async fn get<TRequest: Serialize, TResponse: DeserializeOwned>(&self, path: &str, body: &TRequest) -> Result<TResponse, YoutubeMusicError> {
        let body = YoutubeRequest {
            context: Context::default().as_user(self.user_id.clone()),
            body,
        };
        let mut request = self.client.post(&format!("{URL}{path}"))
            .header("authorization", self.get_authorization());
        if let Some(visitor_id) = self.visitor_id.as_ref() {
            request = request.header("x-goog-visitor-id", visitor_id);
        }
        let response = request
            .query(&YoutubeParameters::default())?
            .body(Body::from_json(&body)?)
            .await?
            .body_json()
            .await?;

        Ok(response)
    }

    fn get_authorization(&self) -> String {
        let auth = format!("{} {YTM_DOMAIN}", self.sapisid);
        let unix_timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut hasher = Sha1::new();
        hasher.update(format!("{unix_timestamp} {auth}"));
        let hash = hasher.finalize();
        let hash = base16ct::lower::encode_string(&hash);

        format!("SAPISIDHASH {}_{}", unix_timestamp, hash)
    }
}

#[derive(Debug, Copy, Clone, Serialize)]
struct YoutubeParameters {
    alt: &'static str,
    key: &'static str,
}

impl Default for YoutubeParameters {
    fn default() -> Self {
        Self {
            alt: "json",
            key: API_KEY,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct YoutubeRequest<'a, TBody> {
    context: Context,
    #[serde(flatten)]
    body: &'a TBody,
}

#[derive(Default, Debug, Clone, Serialize)]
struct Context {
    client: ContextClient,
    user: ContextUser,
}

impl Context {
    fn as_user(mut self, user_id: Option<String>) -> Self {
        self.user.on_behalf_of_user = user_id;

        self
    }
}

#[derive(Default, Debug, Clone, Serialize)]
struct ContextUser {
    on_behalf_of_user: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ContextClient {
    client_name: String,
    client_version: String,
    #[serde(rename = "hl")]
    language: String,
}

impl Default for ContextClient {
    fn default() -> Self {
        Self {
            client_name: "WEB_REMIX".to_string(),
            client_version: "1.20221011.01.00".to_string(),
            language: "en".to_string(),
        }
    }
}

