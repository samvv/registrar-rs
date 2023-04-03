
pub mod json;

#[cfg_attr(feature = "io_error_more", feature("io_error_more"))]

pub mod io_result_ext;

use log::error;
use reqwest::Method;
use serde::{Serialize, Deserialize};
use serde_json::{Value, json, error::Category};
use json::ValueExt;

const DEFAULT_REFRESH_TIMEOUT: u64 = 1;
const DEFAULT_MAX_RETRIES: u32 = 5;

#[derive(Debug)]
pub enum Error {
    AuthenticationFailed,
    UnknownRemoteError(u64),
    Io(std::io::Error),
    Json(String),
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AuthenticationFailed => write!(f, "OpenProvider did not accept the current authentication"),
            Self::UnknownRemoteError(code) => write!(f, "an unrecognised error code {} was returned while contacting OpenProvider", code),
            Self::Io(error) => write!(f, "input/output error: {}", error),
            Self::Json(message) => f.write_str(message),
            Self::Other(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for Error {}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        match error.classify() {
            Category::Io => Error::Io(error.into()),
            Category::Eof | Category::Data | Category::Syntax => Error::Json(error.to_string()),
        }
    }
}

impl From<json::Error> for Error {
    fn from(error: json::Error) -> Self {
        Error::Json(error.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::Other(error.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

struct Config {
    token: Option<String>,
    max_retries: u32,
}

pub struct Builder {
    config: Config,
}

impl Builder {

    pub fn new() -> Self {
        Self {
            config: Config {
                token: None,
                max_retries: DEFAULT_MAX_RETRIES
            }
        }
    }

    pub fn token(mut self, token: Option<String>) -> Self {
        self.config.token = token;
        self
    }

    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.config.max_retries = max_retries;
        self
    }

    pub fn no_max_retries(mut self) -> Self {
        self.config.max_retries = 0;
        self
    }

    pub fn build(&self) -> Client {
        Client {
            client: reqwest::Client::new(),
            token: self.config.token.clone(),
            max_retries: self.config.max_retries,
        }
    }

}

pub struct Client {
    client: reqwest::Client,
    token: Option<String>,
    max_retries: u32,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RecordType {
    A,
    AAAA,
    CAA,
    CNAME,
    MX,
    SPF,
    SRV,
    TXT,
    NS,
    TLSA,
    SSHFP,
    SOA,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Record {
    pub creation_date: Option<String>,
    pub ip: Option<String>,
    pub modification_date: Option<String>,
    pub name: String,
    pub prio: Option<u64>,
    pub ttl: u64,
    #[serde(rename = "type")]
    pub ty: RecordType,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SectigoData {
    pub autorenew: bool,
    pub order_date: String,
    pub renewal_date: String,
    pub securd: bool,
    pub website_id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PremiumDnsData {
    Sectigo(SectigoData),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Zone {
    pub active: bool,
    pub creation_date: String,
    pub dnskey: Option<String>,
    pub id: u64,
    pub ip: String,
    pub is_deleted: bool,
    pub is_shadow: bool,
    pub is_spamexperts_enabled: bool,
    pub modification_date: String,
    pub name: String,
    pub premium_dns: Option<PremiumDnsData>,
    pub provider: String,
    pub records: Option<Vec<Record>>,
    pub reseller_id: u64,
    #[serde(rename = "type")]
    pub ty: String,
}

const CODE_SUCCESS: u64 = 0;

/// The error code the OpenProvider API returns whenever there is an authentication failure.
const CODE_AUTH_FAILED: u64 = 196;

impl Client {

    pub async fn login<S1: AsRef<str>, S2: AsRef<str>>(&mut self, username: S1, password: S2) -> Result<String> {
        let res = self.request(
            Method::POST,
            "https://api.openprovider.eu/v1beta/auth/login",
            Some(serde_json::json!({
                "username": username.as_ref(),
                "password": password.as_ref()
            }))
        ).await?;
        Ok(res.get_ok("token")?.as_str_ok()?.to_string())
    }

    pub fn get_token(&self) -> Option<&String> {
        self.token.as_ref()
    }

    pub fn set_token<S: Into<String>>(&mut self, token: S) {
        self.token = Some(token.into());
    }

    async fn request<U: AsRef<str>>(&mut self, method: Method, url: U, body: Option<Value>) -> Result<Value> {
        let url_ref = url.as_ref();
        log::info!("Starting request to {}", url_ref);
        let mut builder = self.client
            .request(method.clone(), url_ref)
            .header("Accept", "*/*");
        let body = body.clone();
        if body.is_some() {
            builder = builder.json(&body.unwrap());
        }
        if self.token.is_some() {
            builder = builder.header("Authorization", format!("Bearer {}", self.token.clone().unwrap()));
        }
        let response: Value = builder
            .send().await?
            .json().await?;
        let code = response
            .get_ok("code")?
            .as_u64_ok()?;
        if code == CODE_SUCCESS {
            let data = response.get_ok("data")?;
            return Ok(data.clone())
        }
        match code {
            CODE_AUTH_FAILED => Err(Error::AuthenticationFailed),
            _ => Err(Error::UnknownRemoteError(code)),
        }
    }

    pub async fn list_zones(&mut self) -> Result<Vec<Zone>> {
        let response = self.request(
            Method::GET,
            "https://api.openprovider.eu/v1beta/dns/zones",
            Some(json!({}))
        ).await?;
        let zones: std::result::Result<Vec<Zone>, _> = response
            .get_ok("results")?
            .as_array_ok()?
            .iter()
            .map(|x| serde_json::from_value::<Zone>(x.clone()))
            .collect();
        Ok(zones?)
    }

    pub async fn get_zone<S: AsRef<str>>(&mut self, name: S, with_records: bool) -> Result<Zone> {
        let response = self.request(
            Method::GET,
            format!("https://api.openprovider.eu/v1beta/dns/zones/{}?with_records={}", name.as_ref(), if with_records { "true" } else { "false" }),
            None
        ).await?;
        Ok(serde_json::from_value::<Zone>(response)?)
    }

    pub async fn set_record<S: AsRef<str>>(&mut self, name: S, orig_record: Record, new_record: Record) -> Result<()> {
        let name_ref = name.as_ref();
        self.request(
            Method::PUT,
            format!("https://api.openprovider.eu/v1beta/dns/zones/{}", name_ref),
            Some(serde_json::json!({
                "name": name_ref,
                "records":
                {
                    "update":
                    [
                        {
                            "original_record": orig_record,
                            "record": new_record
                        }
                    ]
                }
            }))
        ).await?;
        Ok(())
    }

}

