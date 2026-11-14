// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Client unit tests against the loopback stub; no pain001 server needed.

use super::*;
use crate::stub::{response, serve, serve_raw};
use crate::MessageType;
use serde_json::json;
use std::time::Duration;

#[test]
fn health_decodes_and_sends_no_body() {
    let (url, seen) = serve(
        200,
        r#"{"status":"healthy","version":"0.0.85","message":"ok"}"#,
    );
    let health = Client::new(format!("{url}/")).health().expect("health");
    assert_eq!(health.status, "healthy");
    let seen = seen.recv().expect("request");
    assert_eq!(seen.request_line, "GET /api/v1/health HTTP/1.1");
    assert_eq!(seen.header("authorization"), None);
    assert!(seen.body.is_empty());
}

#[test]
fn every_request_carries_the_user_agent() {
    let (url, seen) = serve(200, "[]");
    Client::new(url).faces().expect("faces");
    let seen = seen.recv().expect("request");
    assert_eq!(seen.header("user-agent"), Some(USER_AGENT));
    assert_eq!(USER_AGENT, format!("pain001-sdk-rust/{VERSION}"));
    assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
}

#[test]
fn the_api_key_travels_as_a_bearer_token_and_never_in_debug() {
    let (url, seen) = serve(200, "[]");
    let client = Client::new(url)
        .api_key("s3cret")
        .timeout(Duration::from_secs(5));
    assert!(client.faces().expect("faces").is_empty());
    assert_eq!(
        seen.recv().expect("request").header("authorization"),
        Some("Bearer s3cret")
    );
    let debug = format!("{client:?}");
    assert!(!debug.contains("s3cret") && debug.contains("<redacted>"));
}

#[test]
fn validate_posts_the_file_payload() {
    let (url, seen) = serve(200, r#"{"is_valid":true,"total_rows":5}"#);
    let client = Client::new(url);
    let result = client
        .validate(
            "id,date",
            "p.csv",
            MessageType::Pain00100109,
            Some("sepa-sct"),
        )
        .expect("validate");
    assert!(result.is_valid);
    let seen = seen.recv().expect("request");
    assert_eq!(seen.request_line, "POST /api/v1/ui/validate HTTP/1.1");
    assert_eq!(seen.header("content-type"), Some("application/json"));
    let body: Value = serde_json::from_str(&seen.body).expect("json body");
    assert_eq!(
        body,
        json!({"content": "id,date", "filename": "p.csv",
               "message_type": "pain.001.001.09", "scheme": "sepa-sct"})
    );
}

#[test]
fn a_string_detail_maps_to_an_api_error() {
    let (url, _seen) = serve(400, r#"{"detail":"not a valid IBAN"}"#);
    let error = Client::new(url)
        .generate("x", "p.csv", "pain.001.001.09", None)
        .expect_err("400");
    assert!(
        matches!(error, Error::Api { status: 400, ref detail } if detail == "not a valid IBAN"),
        "{error:?}"
    );
}

const REFUSAL: &str = r#"{"success":false,"message":"Validation failed","xml":null,
    "validation_errors":[{"field":"$.currency","message":"not ISO 4217","value":"EURO"}],
    "scheme_violations":[{"rule":"UK-CHAPS-PURP","severity":"error","field":"purpose_code"}]}"#;

#[test]
fn a_422_refusal_from_generate_carries_the_typed_findings() {
    let (url, _seen) = serve(422, REFUSAL);
    let error = Client::new(url)
        .generate("x", "p.csv", MessageType::Pain00100109, Some("uk-chaps"))
        .expect_err("422");
    assert_eq!(error.status(), Some(422));
    let refusal = error.refusal().expect("typed refusal");
    assert!(!refusal.success && refusal.xml.is_none());
    assert_eq!(refusal.validation_errors[0].field, "$.currency");
    assert_eq!(refusal.validation_errors[0].value, Some(json!("EURO")));
    assert_eq!(refusal.scheme_violations[0]["rule"], "UK-CHAPS-PURP");
    assert_eq!(
        error.to_string(),
        "pain001 API refused the file (HTTP 422): Validation failed"
    );
}

#[test]
fn a_422_from_generate_without_a_refusal_body_stays_an_api_error() {
    let (url, _seen) = serve(422, r#"{"detail":[{"loc":["body","message_type"]}]}"#);
    let error = Client::new(url)
        .generate("x", "p.csv", "pain.999", None)
        .expect_err("422");
    assert!(error.refusal().is_none());
    assert!(matches!(error, Error::Api { status: 422, ref detail } if detail.is_array()));
    let (url, _seen) = serve(400, REFUSAL);
    let error = Client::new(url)
        .generate("x", "p.csv", "pain.001.001.09", None)
        .expect_err("400");
    assert!(matches!(error, Error::Api { status: 400, .. }), "{error:?}");
}

#[test]
fn a_2xx_generate_without_the_xml_is_a_decode_error() {
    // Found by the `response` fuzz target: the README promises that an
    // `Ok` from generate always carries the XML.
    for body in [
        r#"{"success":true,"message":"Generated"}"#,
        r#"{"success":true,"message":"Generated","xml":null}"#,
        r#"{"success":false,"message":"Validation failed","xml":"<x/>"}"#,
    ] {
        let (url, _seen) = serve(200, body);
        let error = Client::new(url)
            .generate("x", "p.csv", "pain.001.001.09", None)
            .expect_err("no document");
        assert!(matches!(error, Error::Decode(_)), "{body}: {error:?}");
    }
    let (url, _seen) = serve(200, r#"{"success":true,"message":"ok","xml":"<x/>"}"#);
    let generated = Client::new(url)
        .generate("x", "p.csv", "pain.001.001.09", None)
        .expect("document");
    assert_eq!(generated.xml.as_deref(), Some("<x/>"));
}

#[test]
fn an_array_detail_maps_to_an_api_error() {
    let (url, _seen) = serve(422, r#"{"detail":[{"loc":["body","xml"]}]}"#);
    let error = Client::new(url).check("<x/>", None, None).expect_err("422");
    match error {
        Error::Api { status, detail } => {
            assert_eq!(status, 422);
            assert!(detail.is_array());
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn a_non_json_error_body_is_kept_as_text() {
    let (url, _seen) = serve(502, "Bad Gateway");
    let error = Client::new(url).health().expect_err("502");
    assert!(matches!(error, Error::Api { status: 502, ref detail } if detail == "Bad Gateway"));
}

#[test]
fn a_2xx_body_of_the_wrong_shape_is_a_decode_error() {
    let (url, _seen) = serve(200, r#"{"status":1}"#);
    let error = Client::new(url).health().expect_err("decode");
    assert!(matches!(error, Error::Decode(_)), "{error:?}");
}

#[test]
fn request_returns_any_status_without_erroring() {
    let (url, seen) = serve(404, r#"{"detail":"Unknown face"}"#);
    let body = json!({"xml": "<x/>"});
    let (status, value) = Client::new(url)
        .request("POST", "/api/v1/faces/nope", Some(&body))
        .expect("request");
    assert_eq!((status, value), (404, json!({"detail": "Unknown face"})));
    assert_eq!(seen.recv().expect("request").body, r#"{"xml":"<x/>"}"#);
}

#[test]
fn an_empty_body_parses_as_null() {
    let (url, _seen) = serve(204, "");
    let (status, value) = Client::new(url)
        .request("DELETE", "/x", None)
        .expect("request");
    assert_eq!((status, value), (204, Value::Null));
}

#[test]
fn project_face_encodes_the_face_and_sends_bindings() {
    let (url, seen) = serve(
        200,
        r#"{"face":"a b","version":"pain.001.001.09","payload":{"k":1},
            "loss_report":{"face":"a b","total_source_elements":2,"mapped_count":1,
            "unmapped_count":1,"is_lossless":false,"loss_ratio":0.5,
            "mapped_paths":["/a"],"unmapped_paths":["/b"]}}"#,
    );
    let mut bindings = Map::new();
    bindings.insert("ConsentId".into(), json!("c-1"));
    let projected = Client::new(url)
        .project_face("a b/c", "<x/>", Some("pain.001.001.09"), Some(&bindings))
        .expect("project");
    assert_eq!(projected.payload, json!({"k": 1}));
    assert!((projected.loss_report.loss_ratio - 0.5).abs() < f64::EPSILON);
    let seen = seen.recv().expect("request");
    assert_eq!(seen.request_line, "POST /api/v1/faces/a%20b%2Fc HTTP/1.1");
    let body: Value = serde_json::from_str(&seen.body).expect("json body");
    assert_eq!(body["bindings"], json!({"ConsentId": "c-1"}));
}

#[test]
fn a_refused_connection_is_a_transport_error() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let url = format!("http://{}", listener.local_addr().expect("addr"));
    drop(listener);
    let error = Client::new(url).health().expect_err("refused");
    assert!(matches!(error, Error::Transport(_)), "{error:?}");
}

#[test]
fn an_invalid_url_is_a_transport_error() {
    let client = Client::new("not a url");
    assert_eq!(client.base_url(), "not a url");
    let error = client.health().expect_err("invalid");
    assert!(matches!(error, Error::Transport(_)), "{error:?}");
}

#[test]
fn helpers_parse_and_encode() {
    assert_eq!(parse_body("  "), Value::Null);
    assert_eq!(parse_body("{\"a\":1}"), json!({"a": 1}));
    assert_eq!(encode_segment("berlin_group"), "berlin_group");
    assert_eq!(encode_segment("é?"), "%C3%A9%3F");
    assert_eq!(DEFAULT_TIMEOUT, Duration::from_secs(30));
}

#[test]
fn a_body_cut_short_is_a_transport_error() {
    // The stub promises 64 bytes, sends 9 and closes the connection.
    let mut raw = response(200, "");
    raw.truncate(raw.len() - "Content-Length: 0\r\nConnection: close\r\n\r\n".len());
    raw.extend_from_slice(b"Content-Length: 64\r\nConnection: close\r\n\r\n{\"status\"");
    let (url, _seen) = serve_raw(raw);
    let error = Client::new(url).health().expect_err("short body");
    assert!(matches!(error, Error::Transport(_)), "{error:?}");
}

/// A request body whose `Serialize` fails when it holds `false`, which
/// none of the client's own request types can do. One type for both
/// outcomes, so one instantiation of `exchange` covers both.
struct Encodable(bool);

impl Serialize for Encodable {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if self.0 {
            return serializer.serialize_bool(true);
        }
        Err(serde::ser::Error::custom("cannot encode"))
    }
}

#[test]
fn a_body_that_cannot_be_encoded_is_a_decode_error_and_is_not_sent() {
    let (url, seen) = serve(200, "{}");
    let client = Client::new(url);
    let error = client
        .exchange("POST", "/x", Some(&Encodable(false)))
        .expect_err("encode");
    assert!(matches!(error, Error::Decode(_)), "{error:?}");
    // The stub is still waiting: the failed call sent nothing.
    let (status, _) = client
        .exchange("POST", "/x", Some(&Encodable(true)))
        .expect("sent");
    assert_eq!(status, 200);
    assert_eq!(seen.recv().expect("request").body, "true");
}
