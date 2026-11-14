// SPDX-License-Identifier: Apache-2.0 OR MIT
//! The checks the fuzz targets run. Each one states a promise the crate's
//! documentation makes about what a server's reply turns into, and panics
//! (a libFuzzer crash) when the client breaks it.

use std::fmt::Debug;

use pain001_sdk::fuzzing::{decode, generate, parse_body};
use pain001_sdk::{
    CheckResponse, Error, FaceInfo, FaceResponse, GenerateResponse, HealthResponse,
    ValidationResponse,
};
use serde_json::Value;

/// Splits a fuzz input into a status line and a body. `None` unless the
/// first line is a status code `http` accepts (100 to 999).
pub fn split_status(text: &str) -> Option<(u16, &str)> {
    let (head, body) = text.split_once('\n')?;
    let status: u16 = head.trim().parse().ok()?;
    (100..=999).contains(&status).then_some((status, body))
}

/// A 2xx body, decoded as every typed call's response.
pub fn check_success_body(text: &str) {
    check_parse_body(text);
    check_typed(decode::<HealthResponse>(200, text));
    check_typed(decode::<ValidationResponse>(200, text));
    check_typed(decode::<CheckResponse>(200, text));
    check_typed(decode::<Vec<FaceInfo>>(200, text));
    check_typed(decode::<FaceResponse>(200, text));
    check_generated(generate(200, text));
}

/// A reply with any status. 2xx bodies are the `response` target's job.
pub fn check_reply(status: u16, body: &str) {
    if (200..300).contains(&status) {
        return;
    }
    let value = parse_body(body);
    let error = decode::<HealthResponse>(status, body).expect_err("non-2xx is an error");
    check_api(status, &value, &error);
    let error = generate(status, body).expect_err("non-2xx is an error");
    check_generate_error(status, &value, &error);
}

/// `Client::request`: empty is `null`, JSON is itself, anything else is
/// the text as a JSON string.
fn check_parse_body(text: &str) {
    let value = parse_body(text);
    let expected = if text.trim().is_empty() {
        Value::Null
    } else {
        serde_json::from_str(text).unwrap_or_else(|_| Value::String(text.to_owned()))
    };
    assert_eq!(value, expected);
}

/// A 2xx body decodes, or fails with `Error::Decode` and nothing else.
fn check_typed<T: Debug>(result: Result<T, Error>) {
    match result {
        Ok(value) => drop(format!("{value:?}")),
        Err(error) => check_decode_error(&error),
    }
}

/// README: "an `Ok` always carries" the generated XML.
fn check_generated(result: Result<GenerateResponse, Error>) {
    match result {
        Ok(response) => assert!(
            response.success && response.xml.is_some(),
            "Ok without a document: {response:?}"
        ),
        Err(error) => check_decode_error(&error),
    }
}

fn check_decode_error(error: &Error) {
    assert!(matches!(error, Error::Decode(_)), "2xx gave {error:?}");
    assert_eq!(error.status(), None);
    assert!(error.refusal().is_none() && error.request_errors().is_none());
    let shown = error.to_string();
    assert!(shown.starts_with("pain001 API response could not be decoded"));
}

/// `Error::Api`'s `detail`: the body's `detail` field when the body is an
/// object that has one, the whole body otherwise.
fn check_api(status: u16, body: &Value, error: &Error) {
    let Error::Api {
        status: got,
        detail,
    } = error
    else {
        panic!("undocumented variant for HTTP {status}: {error:?}");
    };
    assert_eq!(*got, status);
    let expected = match body {
        Value::Object(map) if map.contains_key("detail") => &map["detail"],
        other => other,
    };
    assert_eq!(detail, expected);
    assert!(error.refusal().is_none());
    if let Some(items) = error.request_errors() {
        assert_eq!(Some(items.len()), detail.as_array().map(Vec::len));
    }
    check_shown(status, error);
}

/// A generate call: `Refused` only for a `422` refusal body, with
/// `success: false` and no XML; `Api` for everything else.
fn check_generate_error(status: u16, body: &Value, error: &Error) {
    match error {
        Error::Refused {
            status: got,
            response,
        } => {
            assert_eq!((*got, status), (422, 422));
            assert!(!response.success, "refusal with success: {response:?}");
            assert!(response.xml.is_none(), "refusal with XML: {response:?}");
            assert!(error.refusal().is_some() && error.request_errors().is_none());
            check_shown(status, error);
        }
        Error::Api { .. } => {
            assert!(
                !is_refusal(status, body),
                "refusal body kept as Api: {body}"
            );
            check_api(status, body, error);
        }
        other => panic!("undocumented variant for HTTP {status}: {other:?}"),
    }
}

/// Error::Refused's contract: a `422` whose body decodes as a
/// `GenerateResponse` with `success: false` and no `xml`.
fn is_refusal(status: u16, body: &Value) -> bool {
    status == 422
        && serde_json::from_value::<GenerateResponse>(body.clone())
            .is_ok_and(|r| !r.success && r.xml.is_none())
}

fn check_shown(status: u16, error: &Error) {
    assert_eq!(error.status(), Some(status));
    let shown = error.to_string();
    assert!(shown.starts_with("pain001 API "), "{shown}");
    assert!(shown.contains(&format!("HTTP {status}")), "{shown}");
    drop(format!("{error:?}"));
}
