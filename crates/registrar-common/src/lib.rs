
mod io_result_ext;
mod json;
mod error;

use http::{Method, Request};
use http_body_util::{BodyExt, Empty};
use hyper::body::Bytes;
use hyper_util::rt::TokioIo;
pub use io_result_ext::*;
pub use json::*;
pub use error::*;
use serde_json::Value;
use tokio::net::TcpStream;

const DEFAULT_MAX_RETRIES: u32 = 5;

#[derive(Clone, Debug)]
pub enum DnsRecordType {
    /// IPv6 host address
    AAAA,
    /// Auto resolved alias
    ALIAS,
    /// Canonical name for an alias
    CNAME,
    /// Mail eXchange
    MX,
    /// Name Server
    NS,
    /// Pointer
    PTR,
    /// Start Of Authority
    SOA,
    /// location of service
    SRV,
    /// Descriptive text
    TXT,
    /// DNSSEC public key
    DNSKEY,
    /// Delegation Signer
    DS,
    /// Next Secure
    NSEC,
    /// Next Secure v. 3
    NSEC3,
    /// NSEC3 Parameters
    NSEC3PARAM,
    /// RRset Signature
    RRSIG,
    /// AFS Data Base location
    AFSDB,
    /// Certification Authority Authorization
    CAA,
    /// Certificate / CRL
    CERT,
    /// DHCP Information
    DHCID,
    /// Non-Terminal DNS Name Redirection
    DNAME,
    /// Host information
    HINFO,
    /// HTTPS Service binding and parameter specification
    HTTPS,
    /// Location information
    LOC,
    /// Naming Authority Pointer
    NAPTR,
    /// Responsible person
    RP,
    /// Transport Layer Security Authentication
    TLSA,
}

#[derive(Clone, Debug)]
pub struct DnsRecord {
    pub name: String,
    pub ty: DnsRecordType,
    pub content: String,
    pub ttl: Option<u32>,
    pub priority: Option<u32>,
}

/// Represents the category of all computations that may fail in this library.
pub type Result<T> = std::result::Result<T, Error>;

pub type DnsRecordId = String;

#[derive(Clone, Debug)]
pub struct PageHint {
    start: u32,
    count: u32,
}

impl PageHint {

    pub fn new(start: u32, count: u32) -> Self {
        Self { start, count }
    }

    pub fn start(&self) -> u32 {
        self.start
    }

    pub fn count(&self) -> u32 {
        self.count
    }

}

pub struct ApiClient {
    base_url: String,
    max_retries: u32,
}

impl ApiClient {

    pub fn new<S: Into<String>>(base_url: S) -> Self {
        Self {
            base_url: base_url.into(),
            max_retries: DEFAULT_MAX_RETRIES,
        }
    }

    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn get<S: AsRef<str>>(&self, path: S) -> ApiRequest {
        ApiRequest::new(
            format!("{}{}", self.base_url, path.as_ref()),
            Method::GET
        )
    }

}

pub struct ApiRequest {
    url: String,
    method: Method,
    bearer: Option<String>,
}

impl ApiRequest {

    fn new(url: String, method: Method) -> Self {
        Self {
            url,
            method,
            bearer: None,
        }
    }

    pub async fn send<S: AsRef<str>>(&self) -> Result<Value> {

        log::info!("{} {}", self.method, self.url);

        let url = self.url.parse::<hyper::Uri>()?;

        let port = match url.port() {
            None => match url.scheme_str() {
                Some("http") => 80,
                Some("https") => 443,
                _ => unreachable!(),
            },
            Some(port) => port.as_u16(),
        };

        let address = format!("{}://{}:{}", url.scheme_str().unwrap(), url.host().unwrap(), port);

        eprintln!("{}", address);

        let stream = TcpStream::connect(address).await?;
        let io = TokioIo::new(stream);

        // Create the Hyper client
        let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;

        // Wait for the connection to be established
        conn.await?;

        // Create an HTTP request with an empty body and a HOST header
        let req = Request::builder()
            .uri(&url)
            .method(self.method.clone())
            .header(hyper::header::HOST, url.authority().ok_or(Error::InvalidUri("authority is missing".to_owned()))?.as_str())
            .body(Empty::<Bytes>::new())?;

        // Await the response...
        let res = sender.send_request(req).await?;

        let status = res.status().as_u16();
        if status < 200 || status > 300 {
            return Err(Error::StatusCode(status));
        }

        let data = res.into_body().collect().await?.to_bytes();
        Ok(serde_json::from_slice(&data)?)
    }

}

pub trait Registrar {

    fn list_records(&mut self, hint: &PageHint) -> Result<Vec<DnsRecord>>;

    fn create_record<S: AsRef<str>>(&mut self, name: S, record: &DnsRecord) -> Result<DnsRecordId>;

}

