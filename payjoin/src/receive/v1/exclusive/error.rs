use core::fmt;
use std::error;

use crate::receive::JsonReply;

/// Error that occurs during validation of an incoming v1 payjoin request.
///
/// This type provides a stable public API for v1 request validation errors while keeping internal
/// error variants private. It handles validation of:
/// - PSBT parsing and validation
/// - I/O operations during request processing
/// - HTTP headers (Content-Type, Content-Length)
///
/// The error messages are formatted as JSON strings according to the BIP-78 spec with appropriate
/// error codes and human-readable messages.
#[derive(Debug)]
pub struct RequestError(InternalRequestError);

#[derive(Debug)]
pub(crate) enum InternalRequestError {
    /// I/O error while reading the request body
    Io(std::io::Error),
    /// A required HTTP header is missing from the request
    MissingHeader(&'static str),
    /// The Content-Type header has an invalid value
    InvalidContentType(String),
    /// The Content-Length header could not be parsed as a number
    InvalidContentLength(std::num::ParseIntError),
    /// The Content-Length value exceeds the maximum allowed size
    ContentLengthTooLarge(usize),
}

impl From<InternalRequestError> for RequestError {
    fn from(value: InternalRequestError) -> Self { RequestError(value) }
}

impl From<InternalRequestError> for super::ReplyableError {
    fn from(e: InternalRequestError) -> Self { super::ReplyableError::V1(e.into()) }
}

impl From<&RequestError> for JsonReply {
    fn from(e: &RequestError) -> Self {
        use InternalRequestError::*;

        use crate::error_codes::ErrorCode::OriginalPsbtRejected;
        match &e.0 {
            Io(_) => JsonReply::new(OriginalPsbtRejected, e),
            MissingHeader(_) => JsonReply::new(OriginalPsbtRejected, e),
            InvalidContentType(_) => JsonReply::new(OriginalPsbtRejected, e),
            InvalidContentLength(_) => JsonReply::new(OriginalPsbtRejected, e),
            ContentLengthTooLarge(_) => JsonReply::new(OriginalPsbtRejected, e),
        }
    }
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            InternalRequestError::Io(e) => write!(f, "{}", e),
            InternalRequestError::MissingHeader(header) => write!(f, "Missing header: {}", header),
            InternalRequestError::InvalidContentType(content_type) =>
                write!(f, "Invalid content type: {}", content_type),
            InternalRequestError::InvalidContentLength(e) => write!(f, "{}", e),
            InternalRequestError::ContentLengthTooLarge(length) =>
                write!(f, "Content length too large: {}.", length),
        }
    }
}

impl error::Error for RequestError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match &self.0 {
            InternalRequestError::Io(e) => Some(e),
            InternalRequestError::InvalidContentLength(e) => Some(e),
            InternalRequestError::MissingHeader(_) => None,
            InternalRequestError::InvalidContentType(_) => None,
            InternalRequestError::ContentLengthTooLarge(_) => None,
        }
    }
}
