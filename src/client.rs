// SPDX-License-Identifier: Apache-2.0 OR MIT
//! The blocking client.

use std::fmt;
use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{Map, Value};
use ureq::http;

use crate::error::{api_error, generate_error, generated, Error};
use crate::types::{
    CheckRequest, CheckResponse, FaceInfo, FaceRequest, FaceResponse, FilePayload,
    GenerateResponse, HealthResponse, ValidationResponse,
};

/// The timeout a new [`Client`] uses for a whole call (connect, send and
/// read), until [`Client::timeout`] changes it.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// This crate's version, from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The `User-Agent` every request sends: `pain001-sdk-rust/<VERSION>`, so a
/// server's access log shows which client and release made a call.
pub const USER_AGENT: &str = concat!("pain001-sdk-rust/", env!("CARGO_PKG_VERSION"));

/// Largest response body read, in bytes. A generated document for the
/// largest accepted input (10 MiB) fits with room to spare.
const BODY_LIMIT: u64 = 64 * 1024 * 1024;

/// A client for one pain001 server.
///
/// Cheap to clone; clones share the connection pool. `Debug` output never
/// shows the API key.
#[derive(Clone)]
pub struct Client {
    base_url: String,
    api_key: Option<String>,
    timeout: Duration,
    agent: ureq::Agent,
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client")
            .field("base_url", &self.base_url)
            .field("api_key", &self.api_key.as_ref().map(|_| "<redacted>"))
            .field("timeout", &self.timeout)
            .finish()
    }
}

impl Client {
    /// A client for the server at `base_url`, for example
    /// `http://127.0.0.1:8000`. A trailing `/` is ignored.
    pub fn new(base_url: impl Into<String>) -> Self {
        let base_url = base_url.into().trim_end_matches('/').to_owned();
        Client {
            base_url,
            api_key: None,
            timeout: DEFAULT_TIMEOUT,
            agent: build_agent(DEFAULT_TIMEOUT),
        }
    }

    /// Sends `Authorization: Bearer <key>` on every call (the server's
    /// `PAIN001_API_KEY`).
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Sets the timeout for a whole call.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self.agent = build_agent(timeout);
        self
    }

    /// The base URL, without a trailing `/`.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// `GET /api/v1/health`.
    pub fn health(&self) -> Result<HealthResponse, Error> {
        self.call("GET", "/api/v1/health", None::<&()>)
    }

    /// `POST /api/v1/ui/validate`: validates a CSV, JSON or JSONL payment
    /// file's text. `filename`'s suffix picks the parser; `scheme` names
    /// rulebooks, comma-separated (for example `sepa-sct`).
    pub fn validate(
        &self,
        content: &str,
        filename: &str,
        message_type: impl AsRef<str>,
        scheme: Option<&str>,
    ) -> Result<ValidationResponse, Error> {
        let body = FilePayload {
            content,
            filename,
            message_type: message_type.as_ref(),
            scheme,
        };
        self.call("POST", "/api/v1/ui/validate", Some(&body))
    }

    /// `POST /api/v1/ui/generate`: turns a payment file's text into XML.
    ///
    /// # Errors
    ///
    /// [`Error::Refused`] when the server refuses the rows or a scheme rule
    /// (HTTP `422`), carrying the typed findings; [`Error::Api`] for any
    /// other non-2xx reply, such as a `400` for an unreadable file;
    /// [`Error::Decode`] for a 2xx reply without `success: true` and the
    /// XML, so an `Ok` always carries the document.
    pub fn generate(
        &self,
        content: &str,
        filename: &str,
        message_type: impl AsRef<str>,
        scheme: Option<&str>,
    ) -> Result<GenerateResponse, Error> {
        let body = FilePayload {
            content,
            filename,
            message_type: message_type.as_ref(),
            scheme,
        };
        self.call_with("POST", "/api/v1/ui/generate", Some(&body), generate_error)
            .and_then(generated)
    }

    /// `POST /api/v1/check`: runs the validation ladder on an existing
    /// pain.001 or pain.008 document, adding scheme profiles and a
    /// Schematron preset (`sepa`, `fednow`, `cbpr`) when given.
    pub fn check(
        &self,
        xml: &str,
        scheme: Option<&str>,
        schematron: Option<&str>,
    ) -> Result<CheckResponse, Error> {
        let body = CheckRequest {
            xml,
            scheme,
            schematron,
        };
        self.call("POST", "/api/v1/check", Some(&body))
    }

    /// `GET /api/v1/faces`: the faces a document can be projected into.
    pub fn faces(&self) -> Result<Vec<FaceInfo>, Error> {
        self.call("GET", "/api/v1/faces", None::<&()>)
    }

    /// `POST /api/v1/faces/{face}`: projects a pain.001 document into
    /// `face`. `version` is detected from the namespace when `None`.
    pub fn project_face(
        &self,
        face: &str,
        xml: &str,
        version: Option<&str>,
        bindings: Option<&Map<String, Value>>,
    ) -> Result<FaceResponse, Error> {
        let path = format!("/api/v1/faces/{}", encode_segment(face));
        let body = FaceRequest {
            xml,
            version,
            bindings,
        };
        self.call("POST", &path, Some(&body))
    }

    /// Sends any request and returns the status with the parsed body,
    /// whatever the status. An empty body parses as `null`; a body that is
    /// not JSON becomes a JSON string. `path` starts with `/`.
    pub fn request(
        &self,
        method: &str,
        path: &str,
        body: Option<&Value>,
    ) -> Result<(u16, Value), Error> {
        self.exchange(method, path, body)
    }

    fn call<B, T>(&self, method: &str, path: &str, body: Option<&B>) -> Result<T, Error>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        self.call_with(method, path, body, api_error)
    }

    /// [`Client::call`] with `on_error` building the error for a non-2xx
    /// status from the parsed body.
    fn call_with<B, T>(
        &self,
        method: &str,
        path: &str,
        body: Option<&B>,
        on_error: fn(u16, Value) -> Error,
    ) -> Result<T, Error>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        let (status, value) = self.exchange(method, path, body)?;
        decode(status, value, on_error)
    }

    /// Encodes `body` as JSON and sends the request.
    fn exchange<B>(&self, method: &str, path: &str, body: Option<&B>) -> Result<(u16, Value), Error>
    where
        B: Serialize + ?Sized,
    {
        let bytes = body
            .map(serde_json::to_vec)
            .transpose()
            .map_err(Error::Decode)?;
        self.send(method, path, bytes)
    }

    fn send(&self, method: &str, path: &str, body: Option<Vec<u8>>) -> Result<(u16, Value), Error> {
        let head = self.head(method, path)?;
        self.once(head, body)
    }

    /// The request line and headers every call shares; an invalid method
    /// or URL fails here, before anything is sent.
    fn head(&self, method: &str, path: &str) -> Result<http::Request<()>, Error> {
        let mut builder = http::Request::builder()
            .method(method)
            .uri(format!("{}{}", self.base_url, path))
            .header("Accept", "application/json")
            .header("User-Agent", USER_AGENT);
        if let Some(key) = &self.api_key {
            builder = builder.header("Authorization", format!("Bearer {key}"));
        }
        builder.body(()).map_err(Error::transport)
    }

    /// Sends `head` once, with `body` as JSON when there is one, and reads
    /// the status and the parsed body.
    fn once(&self, head: http::Request<()>, body: Option<Vec<u8>>) -> Result<(u16, Value), Error> {
        let result = match body {
            Some(bytes) => {
                let (mut parts, ()) = head.into_parts();
                parts.headers.insert(
                    http::header::CONTENT_TYPE,
                    http::HeaderValue::from_static("application/json"),
                );
                self.agent.run(http::Request::from_parts(parts, bytes))
            }
            None => self.agent.run(head),
        };
        let mut response = result.map_err(Error::transport)?;
        let status = response.status().as_u16();
        let text = response
            .body_mut()
            .with_config()
            .limit(BODY_LIMIT)
            .read_to_string()
            .map_err(Error::transport)?;
        Ok((status, parse_body(&text)))
    }
}

fn build_agent(timeout: Duration) -> ureq::Agent {
    ureq::Agent::config_builder()
        .http_status_as_error(false)
        .timeout_global(Some(timeout))
        .build()
        .new_agent()
}

/// Turns a status and its parsed body into the typed result: a 2xx body
/// decodes as `T` (or fails with [`Error::Decode`]), any other status goes
/// to `on_error`.
pub(crate) fn decode<T>(
    status: u16,
    value: Value,
    on_error: fn(u16, Value) -> Error,
) -> Result<T, Error>
where
    T: DeserializeOwned,
{
    if !(200..300).contains(&status) {
        return Err(on_error(status, value));
    }
    serde_json::from_value(value).map_err(Error::Decode)
}

/// Parses a response body: empty is `null`, non-JSON is a JSON string.
pub(crate) fn parse_body(text: &str) -> Value {
    if text.trim().is_empty() {
        return Value::Null;
    }
    serde_json::from_str(text).unwrap_or_else(|_| Value::String(text.to_owned()))
}

/// Percent-encodes one path segment, keeping RFC 3986 unreserved bytes.
fn encode_segment(segment: &str) -> String {
    let mut out = String::with_capacity(segment.len());
    for byte in segment.bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

#[cfg(test)]
mod tests;
