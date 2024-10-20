
use http::{header::{InvalidHeaderName, InvalidHeaderValue, MaxSizeReached}, status::InvalidStatusCode, uri::InvalidUri};
use serde_json::{Number, Value};

use crate::{NumberType, ValueType};

// #[derive(Clone, Debug)]
// pub enum HttpErrorKind {
//     /// Error is from a type Builder
//     Builder,
//     /// The request or response body has issues
//     Body,
//     /// Unparseable response status
//     Status,
//     /// Related to decoding the response's body
//     Decode,
//     /// Broken redirect policy
//     Redirect,
//     /// Connection issues
//     Connect,
//     /// HTTP request issues
//     Request,
//     /// Related to a timeout
//     Timeout,
//     /// Other kinds that aren't directly handled by this library
//     Unknown,
// }

#[derive(Debug)]
pub enum HttpErrorKind {
    Parse,
    ParseTooLarge,
    ParseStatus,
    User,
    Canceled,
    Closed,
    IncCmpleteMessage,
    BodyWriteAborted,
    Timeout,
    Unknown,
}

#[derive(Debug)]
pub enum Error {

    /// Indicates that authentication with the API failed.
    AuthenticationFailed,

    /// No path was provided to a builder while making a request.
    MissingPath,

    /// The API it returned an error code.
    Api {
        code: Option<u32>,
        message: String,
    },

    /// There was an error while performing the HTTP request.
    Http {
        kind: HttpErrorKind,
        message: String,
    },

    /// An error value indicating that the HTTP response status was not successful.
    StatusCode(u16),

    /// A possible HTTP error value when converting a `StatusCode` from a `u16` or `&str`
    ///
    /// This error indicates that the supplied input was not a valid number, was less
    /// than 100, or was greater than 999.
    ///
    /// This variant originated from the [`http`] crate.
    ParseStatusCode,

    /// A possible HTTP error value when converting `Method` from bytes.
    ///
    /// This variant originated from the [`http`] crate.
    Method,

    /// A possible HTTP error when converting a `HeaderName` from another type.
    ///
    /// This variant originated from the [`http`] crate.
    HeaderName,

    /// A possible HTTP error when converting a `HeaderValue` from a string or byte
    /// slice.
    ///
    /// This variant originated from the [`http`] crate.
    HeaderValue,

    /// HTTP error returned when max capacity of `HeaderMap` is exceeded
    ///
    /// This variant originated from the [`http`] crate.
    MaxSizeReached,

    /// All errors related to the parsing of an URI.
    ///
    /// This variant contains the error message that was generated during the parse of the URI.
    InvalidUri(String),

    /// Any kind of IO error.
    ///
    /// Currently, errors in the network communication are stored in [`Other`](Self::Other) instead of this field.
    /// This is due to an incompatibility with the underlying HTTP library and will be fixed in the
    /// future.
    Io(std::io::Error),

    /// Trying to index a JSON-array with an index that is too large.
    IndexOutOfBounds(usize),

    /// Trying to access an entry in a JSON-object that does not exist.
    KeyMissing(String),

    /// Expected a JSON-value of a different type than what was found.
    WrongType(Value, ValueType),

    /// Expected a JSON-number of a different type than what was found.
    WrongNumberType(Number, NumberType),

    /// Any other error encountered while doing (de)serialization, including parsing JSON strings
    /// and converting to/from structs.
    OtherJson(serde_json::Error),

    /// Any other error stored as a human-readable error message.
    Generic(String),
}

impl std::error::Error for Error {}


impl std::fmt::Display for Error {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AuthenticationFailed => write!(f, "failed to authenticate with API"),
            Self::Http { message, .. } => write!(f, "{}", message),
            Self::Api { code, message } => {
                write!(f, "API returned with error: {}", message)?;
                if let Some(code) = code {
                    write!(f, " (code {})", code)?;
                }
                Ok(())
            },
            Self::Io(error) => write!(f, "input/output error: {}", error),
            Self::KeyMissing(key) => write!(f, "key '{}' is missing in JSON-value", key),
            Self::WrongType(_value, expected) => write!(f, "expected {} but got another JSON-value", expected),
            Self::WrongNumberType(_value, expected) => write!(f, "expected {} but got another type of number", expected),
            Self::IndexOutOfBounds(index) => write!(f, "index {} is out of bounds in JSON-array", index),
            Self::OtherJson(error) => write!(f, "{}", error),
            Self::Generic(message) => write!(f, "{}", message),
            _ => todo!(),
        }
    }

}

impl From<std::io::Error> for Error {

    fn from(error: std::io::Error) -> Self {
        Error::Io(error)
    }

}

impl From<http::Error> for Error {

    fn from(value: http::Error) -> Self {
        if value.is::<MaxSizeReached>() {
            Error::MaxSizeReached
        } else if value.is::<InvalidHeaderName>() {
            Error::HeaderName
        } else if value.is::<InvalidHeaderValue>() {
            Error::HeaderValue
        } else if value.is::<InvalidUri>() {
            Error::InvalidUri(value.get_ref().to_string())
        } else if value.is::<InvalidStatusCode>() {
            Error::ParseStatusCode
        } else {
            Error::Generic("unknown HTTP error".to_owned())
        }
    }

}

impl From<serde_json::Error> for Error {

    fn from(error: serde_json::Error) -> Self {
        match error.classify() {
            serde_json::error::Category::Io => Error::Io(error.into()),
            _ => Error::OtherJson(error),
        }
    }

}

impl From<InvalidUri> for Error {

    fn from(error: InvalidUri) -> Self {
        Error::InvalidUri(error.to_string())
    }

}

impl From<hyper::Error> for Error {

    fn from(error: hyper::Error) -> Self {
        let kind = if error.is_parse() {
            HttpErrorKind::Parse
        } else if error.is_parse_status() {
          HttpErrorKind::ParseStatus
        } else if error.is_user() {
        HttpErrorKind::User
        } else if error.is_canceled() {
          HttpErrorKind::Canceled
        } else if error.is_closed() {
          HttpErrorKind::Closed
        } else if error.is_incomplete_message() {
          HttpErrorKind::IncCmpleteMessage
        } else if error.is_body_write_aborted() {
          HttpErrorKind::BodyWriteAborted
        } else if error.is_timeout() {
          HttpErrorKind::Timeout
        } else {
            HttpErrorKind::Unknown
        };
        Error::Http {
            kind,
            message: error.to_string(),
        }
    }

}

// impl From<reqwest::Error> for Error {

//     fn from(error: reqwest::Error) -> Self {
//         let kind = if error.is_body() {
//             HttpErrorKind::Body
//         } else if error.is_request() {
//             HttpErrorKind::Request
//         } else if error.is_status() {
//             HttpErrorKind::Status
//         } else if error.is_decode() {
//             HttpErrorKind::Decode
//         } else if error.is_builder() {
//             HttpErrorKind::Builder
//         } else if error.is_timeout() {
//             HttpErrorKind::Timeout
//         } else if error.is_connect() {
//             HttpErrorKind::Connect
//         } else if error.is_redirect() {
//             HttpErrorKind::Redirect
//         } else {
//             HttpErrorKind::Unknown
//         };
//         Error::Http { kind, message: error.to_string() }
//     }

// }
