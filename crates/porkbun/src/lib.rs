
mod json;

use json::ValueExt;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::{error::Category, Value};

const BASE_URL: &str = "https://api.porkbun.com/api/json/v3";

#[derive(Debug)]
pub enum Error {
    AuthenticationFailed,
    /// This is due to an incompatibility with the underlying HTTP library and will be fixed in the
    /// future.
    Io(std::io::Error),
    /// Data sent or received from OpenProvider did not meet the schema of this library.
    Json(String),
    /// Any other error stored as a human-readable error message.
    Other(String),
}


impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AuthenticationFailed => write!(f, "OpenProvider did not accept the current authentication"),
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

pub struct Client {
    client: reqwest::Client,
    keys: Option<(String, String)>,
}

pub struct Builder {
    max_retries: u32,
    api_key: Option<String>,
    secret_api_key: Option<String>,
}

impl Builder {

    pub fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    pub fn api_key<S: Into<String>>(mut self, key: S) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn secret_api_key<S: Into<String>>(mut self, key: S) -> Self {
        self.secret_api_key = Some(key.into());
        self
    }

    pub fn build(self) -> Client {
        Client {
            client: reqwest::Client::new(),
            keys: Some((self.api_key.expect("API key must be set"), self.secret_api_key.expect("secret API key must be set"))),
        }
    }

}

#[derive(Serialize, Deserialize)]
enum DnsRecordType {
    A, MX, CNAME, ALIAS, TXT, NS, AAAA, SRV, TLSA, CAA, HTTPS, SVCB
}

#[derive(Serialize, Deserialize)]
struct DnsRecord {
    #[serde(rename = "secretapikey")]
    secret_api_key: String,
    #[serde(rename = "apikey")]
    api_key: String,
    name: String,
    #[serde(rename = "type")]
    ty: DnsRecordType,
    content: String,
    ttl: Option<u32>,
    prio: Option<u32>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "status")]
enum CreateDnsRecordResponse {
    #[serde(rename = "ERROR")]
    Error {
        message: String,
    },
    #[serde(rename = "SUCCESS")]
    Success {
        id: String,
    }
}

impl Client {

    async fn request<U: AsRef<str>>(&mut self, method: Method, url: U, mut body: Value) -> Result<Value> {
        let url = format!("{}{}", BASE_URL, url.as_ref());
        log::info!("Starting request to {}", url);
        let mut builder = self.client
            .request(method.clone(), url)
            .header("Accept", "*/*");
        let obj = body.as_object_mut().unwrap();
        if let Some((key, secret_key)) = &self.keys {
            obj.insert("apikey".to_owned(), key.clone().into());
            obj.insert("secretapikey".to_owned(), secret_key.clone().into());
        }
        builder = builder.json(&body);
        let response: Value = builder
            .send().await?
            .json().await?;
        if response.get_ok("status")?.as_str_ok()? != "SUCCESS" {
            Err(Error::Other(response.get_ok("message")?.as_str_ok()?.to_owned()))
        } else {
            Ok(response)
        }
    }

    /// Create a DNS record.
    ///
    /// Returns the ID of the newly created record.
    pub async fn create_dns_record(&mut self, record: &DnsRecord) -> Result<String> {
        Ok(self.request(
            Method::POST,
            "/dns/edit/{name}/{id}",
            serde_json::to_value(record)?
        ).await?.get_ok("id")?.as_str_ok()?.to_owned())
    }

}
