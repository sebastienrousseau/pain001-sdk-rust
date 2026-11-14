// SPDX-License-Identifier: Apache-2.0 OR MIT
//! The request encoding and response decoding paths without the network,
//! for the cargo-fuzz targets in `fuzz/` and the benches in `benches/`.
//! Compiled only under `cfg(fuzzing)` (which cargo-fuzz sets), with the
//! `internals` feature, and for unit tests, so it is not part of the
//! public API of a normal build.

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::client;
use crate::error::{api_error, generate_error, generated};
use crate::types::{CheckRequest, FaceRequest, FilePayload};
use crate::{Error, GenerateResponse};

/// The JSON body [`crate::Client::validate`] and
/// [`crate::Client::generate`] send.
pub fn file_body(
    content: &str,
    filename: &str,
    message_type: &str,
    scheme: Option<&str>,
) -> Result<Vec<u8>, Error> {
    encoded(&FilePayload {
        content,
        filename,
        message_type,
        scheme,
    })
}

/// The JSON body [`crate::Client::check`] sends.
pub fn check_body(
    xml: &str,
    scheme: Option<&str>,
    schematron: Option<&str>,
) -> Result<Vec<u8>, Error> {
    encoded(&CheckRequest {
        xml,
        scheme,
        schematron,
    })
}

/// The JSON body [`crate::Client::project_face`] sends.
pub fn face_body(
    xml: &str,
    version: Option<&str>,
    bindings: Option<&Map<String, Value>>,
) -> Result<Vec<u8>, Error> {
    encoded(&FaceRequest {
        xml,
        version,
        bindings,
    })
}

/// The path [`crate::Client::project_face`] posts to.
pub fn face_path(face: &str) -> String {
    client::face_path(face)
}

fn encoded<B: Serialize>(body: &B) -> Result<Vec<u8>, Error> {
    client::encode(Some(body)).map(Option::unwrap_or_default)
}

/// What the client makes of a response body's text: empty is `null`,
/// non-JSON is a JSON string. [`crate::Client::request`] returns this.
pub fn parse_body(text: &str) -> Value {
    client::parse_body(text)
}

/// What a typed call other than `generate` returns for `status` and the
/// body `text`, as if the server had sent them: a 2xx body decodes as
/// `T`, any other status becomes [`Error::Api`].
pub fn decode<T: DeserializeOwned>(status: u16, text: &str) -> Result<T, Error> {
    client::decode(status, client::parse_body(text), api_error)
}

/// What [`crate::Client::generate`] returns for `status` and the body
/// `text`: as [`decode`], except that a `422` refusal body becomes
/// [`Error::Refused`].
pub fn generate(status: u16, text: &str) -> Result<GenerateResponse, Error> {
    client::decode(status, client::parse_body(text), generate_error).and_then(generated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HealthResponse;
    use serde_json::json;

    #[test]
    fn decode_follows_the_client_mapping() {
        let ok: HealthResponse =
            decode(200, r#"{"status":"healthy","version":"0","message":""}"#).expect("decodes");
        assert_eq!(ok.status, "healthy");
        let refused = generate(422, r#"{"success":false,"message":"m"}"#).expect_err("refused");
        assert!(refused.refusal().is_some());
        let plain = decode::<GenerateResponse>(422, r#"{"success":false,"message":"m"}"#)
            .expect_err("api error");
        assert_eq!((plain.status(), plain.refusal()), (Some(422), None));
        let empty = decode::<HealthResponse>(204, "").expect_err("decode");
        assert_eq!(
            empty.to_string(),
            "pain001 API response could not be decoded: \
             invalid type: null, expected struct HealthResponse"
        );
        assert_eq!(parse_body("not json"), json!("not json"));
    }

    #[test]
    fn the_request_bodies_match_what_the_client_sends() {
        let file = file_body("a", "p.csv", "pain.001.001.09", None).expect("encodes");
        assert_eq!(
            serde_json::from_slice::<Value>(&file).expect("json"),
            json!({"content": "a", "filename": "p.csv", "message_type": "pain.001.001.09"})
        );
        let check = check_body("<x/>", Some("sepa-sct"), Some("sepa")).expect("encodes");
        assert_eq!(
            serde_json::from_slice::<Value>(&check).expect("json"),
            json!({"xml": "<x/>", "scheme": "sepa-sct", "schematron": "sepa"})
        );
        let face = face_body("<x/>", None, Some(&Map::new())).expect("encodes");
        assert_eq!(
            String::from_utf8(face).expect("utf-8"),
            r#"{"xml":"<x/>","bindings":{}}"#
        );
        assert_eq!(face_path("uk open"), "/api/v1/faces/uk%20open");
    }
}
