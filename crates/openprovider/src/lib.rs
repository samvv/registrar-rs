//!
//! [OpenProvider](https://openprovider.com) is a domain registrar based in the Netherlands.
//! The service features a public API that anyone can make use of.
//!
//! This crate implements a subset of that API in Rust. With it, you can query, filter and
//! manipulate DNS records.
//!
//! Unforunately, this crate is not complete yet. Many more APIs, such as SSL certificates, have
//! yet to be implemented. You are invited to try out the API and contribute to the project [back
//! on GitHub](https://github.com/samvv/openprovider-rs).

#[doc(hidden)]
pub mod json;

#[cfg_attr(feature = "io_error_more", feature("io_error_more"))]

#[doc(hidden)]
pub mod io_result_ext;

use reqwest::Method;
use serde::{Serialize, Deserialize};
use serde_json::{Value, json, error::Category};
use json::ValueExt;

const DEFAULT_MAX_RETRIES: u32 = 5;

#[derive(Debug)]
pub enum Error {
    /// Indicates that authentication with the OpenProvider API failed, either because of an
    /// invalid username/password combination or a token that expired.
    AuthenticationFailed,
    /// There was some communication with the OpenProvider API but it returned an error code that is
    /// not understood by this library. The code and the message are saved inside the tuple
    /// elements.
    UnknownRemoteError(u64, String),
    /// Any kind of IO error.
    ///
    /// Currently, errors in the network communication are stored in [`Other`](Self::Other) instead of this field.
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
            Self::UnknownRemoteError(code, desc) => write!(f, "OpenProvider returned with error {}: {}", code, desc),
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

/// Represents the category of all computations that may fail in this library.
///
/// ```no_run
/// use openprovider::Error;
///
/// let mut client = openprovider::Client::default();
///
/// loop {
///     match client.get_zone("example.com").await {
///         Ok(info) => break info,
///         Err(Error::AuthenticationFailed) => {
///             let token = client.login("bob", "123456789").await?;
///             client.set_token(token);
///         },
///         Err(error) => panic!("Failed to fetch DNS zone info: {}", error),
///     }
/// }
/// ```
pub type Result<T> = std::result::Result<T, Error>;

struct Config {
    token: Option<String>,
    max_retries: u32,
}

/// Constructs an [API client](Client).
///
/// Right now, this builder does not accept any options, but more may be added in the future.
///
/// ```no_run
/// let mut client = openprovider::Builder::new().build();
/// // use the client to make requests
/// ```
pub struct Builder {
    config: Config,
}

impl Builder {

    /// Create a new API client builder object.
    pub fn new() -> Self {
        Self {
            config: Config {
                token: None,
                max_retries: DEFAULT_MAX_RETRIES
            }
        }
    }

    /// Make sure the client to be built is configured to use this token.
    pub fn token(mut self, token: Option<String>) -> Self {
        self.config.token = token;
        self
    }

    /// Limit the amount of HTTP request retries to the given number.
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.config.max_retries = max_retries;
        self
    }

    /// Allow as many HTTP request retries as needed in the API client.
    pub fn no_max_retries(mut self) -> Self {
        self.config.max_retries = 0;
        self
    }

    /// Build the actual API client. This is a destructive operation.
    pub fn build(self) -> Client {
        Client {
            client: reqwest::Client::new(),
            token: self.config.token,
            max_retries: self.config.max_retries,
        }
    }

}

/// Represents a DNS record type, such as an A record or MX record.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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

/// Represents a DNS record.
#[derive(Clone, Debug, Serialize, Deserialize)]
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

/// Represents additional data about premium Sectigo DNS services for a [DNS zone](Zone).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SectigoData {
    pub autorenew: bool,
    pub order_date: String,
    pub renewal_date: String,
    pub securd: bool,
    pub website_id: u64,
}

/// Represents additional data about premium DNS services for a [DNS zone](Zone).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PremiumDnsData {
    Sectigo(SectigoData),
}

/// Represents the DNS configuration of a single domain.
#[derive(Clone, Debug, Serialize, Deserialize)]
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

/// Communiates with the OpenProvider.nl API.
///
/// ```no_run
/// let mut client = openprovider::Client::default();
/// let token = client.login("bob", "123456789").await?;
/// client.set_token(token);
/// ```
///
pub struct Client {
    client: reqwest::Client,
    token: Option<String>,
    max_retries: u32,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
            token: None,
            max_retries: DEFAULT_MAX_RETRIES
        }
    }
}

impl Client {

    /// Authenticate with the OpenProvider API and receive a fresh token.
    ///
    /// Use [`set_token`](Self::set_token()) to assign the token to the client that should use it.
    ///
    /// ```no_run
    /// let mut client = openprovider::Client::default();
    ///
    /// let token = client.login("bob", "123456789").await?;
    ///
    /// client.set_token(token);
    /// ```
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

    /// Get the current token used for authorization, if any.
    pub fn get_token(&self) -> Option<&String> {
        self.token.as_ref()
    }

    /// Return `true` if a token is present and ready to be used for authorization; `false`
    /// otherwise.
    pub fn has_token(&self) -> bool {
        self.token.is_some()
    }

    //// Set the token that will be used to authenticate.
    ///
    /// Use [`login`](Self::login()) to obtain a token from a combination of a username and password.
    ///
    /// ```no_run
    /// let mut client = openprovider::Client::default();
    ///
    /// match std::env::var("OPENPROVIDER_TOKEN") {
    ///     Ok(token) => client.set_token(token),
    ///     Err(_) => {},
    /// }
    ///
    /// ```
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
            _ => Err(Error::UnknownRemoteError(code, response.get_ok("desc")?.to_string())),
        }
    }

    /// List all known DNS zones for this particular authenticated user.
    ///
    /// ```no_run
    /// let mut client = openprovider::Client::default();
    ///
    /// // ...
    ///
    /// let zones = client
    ///     .list_zones()
    ///     .await?
    ///     .iter()
    ///     .filter(|z| !z.is_deleted);
    /// ```
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

    async fn get_zone_internal<S: AsRef<str>>(&mut self, name: S, with_records: bool) -> Result<Zone> {
        let response = self.request(
            Method::GET,
            format!("https://api.openprovider.eu/v1beta/dns/zones/{}?with_records={}", name.as_ref(), if with_records { "true" } else { "false" }),
            None
        ).await?;
        Ok(serde_json::from_value::<Zone>(response)?)
    }

    /// Get more information about a specific DNS zone.
    ///
    /// ```no_run
    /// let client = openprovider::Client::default();
    ///
    /// let info = client.get_zone("example.com").await?;
    ///
    /// eprintln!("Zone created on {}", info.creation_date);
    /// eprintln!("Zone modified on {}", info.modification_date);
    /// ```
    pub async fn get_zone<S: AsRef<str>>(&mut self, name: S) -> Result<Zone> {
        self.get_zone_internal(name, false).await
    }

    /// List all records that belong to the provided DNS zone.
    ///
    /// ```no_run
    /// use openprovider::RecordType;
    ///
    /// let client = openprovider::Client::default();
    ///
    /// let records = client.list_records("example.com").await?;
    ///
    /// for record in records {
    ///     if record.name == "wiki" && record.ty == RecordType::A {
    ///         eprintln!("Found our wiki A-record pointing to {}", record.value);
    ///     }
    /// }
    /// ```
    pub async fn list_records<S: AsRef<str>>(&mut self, name: S) -> Result<Vec<Record>> {
        let name_ref = name.as_ref();
        let records = self.get_zone_internal(name_ref, true)
            .await?
            .records
            .unwrap()
            .iter_mut()
            .map(|r| Record {
                creation_date: r.creation_date.clone(),
                ip: r.ip.clone(),
                modification_date: r.modification_date.clone(),
                name:
                    if r.name.len() == name_ref.len() {
                        r.name.clone()
                    } else {
                        r.name.chars().take(r.name.len() - name_ref.len() - 1).collect()
                    },
                prio: r.prio,
                ttl: r.ttl,
                ty: r.ty.clone(),
                value: r.value.clone(),
            })
            .collect();
        Ok(records)
    }

    /// Update a given DNS record with new attributes.
    ///
    /// Due to the way the OpenProvider API works, you must supply the old DNS record as well.
    /// You can do this by using [`list_zones`](Self::list_zones()) and filtering on the DNS record that you want to
    /// change.
    ///
    /// ```
    /// use openprovider::RecordType;
    ///
    /// let record = client.list_record("example.com")
    ///     .await?
    ///     .iter()
    ///     .filter(|r| r.name === "wiki" && r.ty == RecordType::A)
    ///     .first()
    ///     .expect("A record for wiki.example.com not found");
    /// 
    /// let mut new_record = record.clone();
    /// new_record.value = "93.184.216.34".to_string();
    ///
    /// client.set_record("example.com", record, new_record)
    /// ```
    pub async fn set_record<S: AsRef<str>>(&mut self, name: S, orig_record: &Record, new_record: &Record) -> Result<()> {
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

