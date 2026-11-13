// SPDX-License-Identifier: Apache-2.0 OR MIT
//! The conformance kit runner.
//!
//! Every case in `conformance/cases.json` goes through
//! [`Client::request`] against the server at `PAIN001_BASE_URL`; the test
//! is skipped, with a message, when the variable is unset. The runner is a
//! port of core's `tests/test_conformance_kit.py`, and the tests below it
//! need no server: they pin the JSON Pointer lookup, `$file` resolution,
//! the failure messages and the kit's SHA-256 sums.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use pain001_sdk::Client;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};

fn kit() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("conformance")
}

fn cases() -> Vec<Value> {
    let text = fs::read_to_string(kit().join("cases.json")).expect("read cases.json");
    let document: Value = serde_json::from_str(&text).expect("cases.json parses");
    document["cases"].as_array().expect("a cases array").clone()
}

/// Replaces `{"$file": ...}` with that fixture's text, recursively.
fn resolve(body: &Value) -> Value {
    match body {
        Value::Object(map) if map.len() == 1 && map.contains_key("$file") => {
            let name = map["$file"].as_str().expect("$file is a string");
            let text = fs::read_to_string(kit().join(name)).expect("read fixture");
            Value::String(text)
        }
        Value::Object(map) => {
            let resolved: Map<String, Value> =
                map.iter().map(|(k, v)| (k.clone(), resolve(v))).collect();
            Value::Object(resolved)
        }
        Value::Array(items) => Value::Array(items.iter().map(resolve).collect()),
        other => other.clone(),
    }
}

/// RFC 6901 lookup with the reference runner's semantics: `""` is the
/// whole document, an array index is all digits and in range.
fn pointer<'a>(document: &'a Value, pointer: &str) -> Option<&'a Value> {
    let mut value = document;
    for raw in pointer.split('/').skip(1) {
        value = step(value, &raw.replace("~1", "/").replace("~0", "~"))?;
    }
    Some(value)
}

/// One pointer step: an all-digit in-range index, or an object key.
fn step<'a>(value: &'a Value, token: &str) -> Option<&'a Value> {
    match value {
        Value::Array(items) => {
            let digits = !token.is_empty() && token.bytes().all(|b| b.is_ascii_digit());
            digits
                .then(|| token.parse::<usize>().ok())
                .flatten()
                .and_then(|i| items.get(i))
        }
        Value::Object(map) => map.get(token),
        _ => None,
    }
}

/// Deep equality where numbers compare by value, as Python's `==` does
/// for `1` and `1.0`.
fn json_eq(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => a.as_f64() == b.as_f64(),
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(x, y)| json_eq(x, y))
        }
        (Value::Object(a), Value::Object(b)) => {
            a.len() == b.len()
                && a.iter()
                    .all(|(k, v)| b.get(k).is_some_and(|w| json_eq(v, w)))
        }
        _ => left == right,
    }
}

fn has_type(value: &Value, kind: &str) -> bool {
    match kind {
        "string" => value.is_string(),
        "number" => value.is_number(),
        "integer" => value.is_i64() || value.is_u64(),
        "boolean" => value.is_boolean(),
        "array" => value.is_array(),
        "object" => value.is_object(),
        "null" => value.is_null(),
        _ => false,
    }
}

fn entries<'a>(expect: &'a Value, key: &str) -> Vec<(&'a String, &'a Value)> {
    expect
        .get(key)
        .and_then(Value::as_object)
        .map(|map| map.iter().collect())
        .unwrap_or_default()
}

/// The failed expectations for one response (empty means the case passed).
fn check_response(expect: &Value, status: u16, document: &Value) -> Vec<String> {
    let wanted = expect["status"].as_u64().expect("expect.status");
    if u64::from(status) != wanted {
        return vec![format!("status {status} != {wanted}")];
    }
    let mut failures = Vec::new();
    for (ptr, value) in entries(expect, "equals") {
        failures.extend(equals_failure(document, ptr, value));
    }
    for (ptr, kind) in entries(expect, "types") {
        failures.extend(type_failure(
            document,
            ptr,
            kind.as_str().expect("a type name"),
        ));
    }
    for (ptr, needles) in entries(expect, "contains") {
        failures.extend(contains_failures(document, ptr, needles));
    }
    failures
}

fn equals_failure(document: &Value, ptr: &str, wanted: &Value) -> Option<String> {
    match pointer(document, ptr) {
        None => Some(format!("{ptr} missing")),
        Some(found) if !json_eq(found, wanted) => Some(format!("{ptr} != {wanted}")),
        Some(_) => None,
    }
}

fn type_failure(document: &Value, ptr: &str, kind: &str) -> Option<String> {
    match pointer(document, ptr) {
        None => Some(format!("{ptr} missing")),
        Some(found) if !has_type(found, kind) => Some(format!("{ptr} is not {kind}")),
        Some(_) => None,
    }
}

fn contains_failures(document: &Value, ptr: &str, needles: &Value) -> Vec<String> {
    let Some(found) = pointer(document, ptr) else {
        return vec![format!("{ptr} missing")];
    };
    let Some(text) = found.as_str() else {
        return vec![format!("{ptr} is not a string")];
    };
    needles
        .as_array()
        .expect("a list of strings")
        .iter()
        .filter_map(Value::as_str)
        .filter(|needle| !text.contains(needle))
        .map(|needle| format!("{ptr} lacks {}", Value::from(needle)))
        .collect()
}

/// Sends one case through the SDK and returns its failed expectations.
fn run_case(client: &Client, case: &Value) -> Vec<String> {
    let body = case.get("body").map(resolve);
    let method = case["method"].as_str().expect("method");
    let path = case["path"].as_str().expect("path");
    match client.request(method, path, body.as_ref()) {
        Ok((status, document)) => check_response(&case["expect"], status, &document),
        Err(error) => vec![format!("request failed: {error}")],
    }
}

#[test]
fn every_case_holds_for_the_live_server() {
    let Ok(base_url) = std::env::var("PAIN001_BASE_URL") else {
        eprintln!("skipped: set PAIN001_BASE_URL to run the conformance kit");
        return;
    };
    let client = Client::new(base_url);
    let cases = cases();
    let mut failed = Vec::new();
    for case in &cases {
        let id = case["id"].as_str().expect("id");
        let failures = run_case(&client, case);
        println!(
            "case {id}: {}",
            if failures.is_empty() { "pass" } else { "FAIL" }
        );
        if !failures.is_empty() {
            failed.push(format!("{id}: {}", failures.join("; ")));
        }
    }
    println!(
        "{} of {} cases passed",
        cases.len() - failed.len(),
        cases.len()
    );
    assert!(failed.is_empty(), "failed cases:\n{}", failed.join("\n"));
}

#[test]
fn the_kit_has_cases() {
    let cases = cases();
    assert!(!cases.is_empty());
    assert!(cases
        .iter()
        .all(|c| c["id"].is_string() && c["expect"]["status"].is_u64()));
}

#[test]
fn the_runner_reports_each_kind_of_failure() {
    let health = json!({"status": "healthy", "version": "0.0.85", "message": "ok"});
    let expect = json!({
        "status": 200,
        "equals": {"/status": "down", "/nope": 1},
        "types": {"/status": "integer", "/gone": "string"},
        "contains": {"/status": ["zzz"], "/version": ["0.0."], "/none": ["a"]}
    });
    let failures = check_response(&expect, 200, &health);
    for wanted in [
        "/status != \"down\"",
        "/nope missing",
        "/status is not integer",
        "/gone missing",
        "/status lacks \"zzz\"",
        "/none missing",
    ] {
        assert!(
            failures.iter().any(|f| f == wanted),
            "{wanted} not in {failures:?}"
        );
    }
    assert_eq!(failures.len(), 6);
    assert_eq!(
        check_response(&json!({"status": 404}), 200, &health),
        ["status 200 != 404"]
    );
    let faces = json!([{"name": "berlin_group"}]);
    let expect = json!({
        "status": 200,
        "equals": {"/9/name": "x", "/a": 1},
        "contains": {"/0": ["x"]}
    });
    assert_eq!(
        check_response(&expect, 200, &faces),
        ["/9/name missing", "/a missing", "/0 is not a string"]
    );
}

#[test]
fn equals_and_types_follow_the_reference_semantics() {
    let document = json!({"n": 5, "f": 1.5, "b": true, "z": null, "o": {}, "l": [1.0]});
    let expect = json!({
        "status": 200,
        "equals": {"/n": 5.0, "/l": [1], "": document.clone()},
        "types": {"/n": "integer", "/f": "number", "/b": "boolean",
                  "/z": "null", "/o": "object", "/l": "array"}
    });
    assert!(check_response(&expect, 200, &document).is_empty());
    assert!(!has_type(&json!(1.5), "integer"));
    assert!(!has_type(&json!(true), "number"));
    assert!(!has_type(&json!("x"), "unknown"));
    assert!(!json_eq(&json!({"a": 1}), &json!({"b": 1})));
}

#[test]
fn json_pointer_follows_rfc_6901() {
    let document = json!({"a/b": {"~": 1}, "list": [10, 20], "": 3});
    assert_eq!(pointer(&document, "/a~1b/~0"), Some(&json!(1)));
    assert_eq!(pointer(&document, ""), Some(&document));
    assert_eq!(pointer(&document, "/"), Some(&json!(3)));
    assert_eq!(pointer(&document, "/list/1"), Some(&json!(20)));
    assert_eq!(pointer(&document, "/list/2"), None);
    assert_eq!(pointer(&document, "/list/-"), None);
    assert_eq!(pointer(&document, "/list/"), None);
    assert_eq!(pointer(&document, "/list/0/x"), None);
    assert_eq!(pointer(&document, "/missing"), None);
}

#[test]
fn file_references_resolve_to_fixture_text() {
    let body = json!([{"$file": "fixtures/bad-iban.pain.001.001.09.csv"}, 1]);
    let resolved = resolve(&body);
    assert!(resolved[0].as_str().expect("text").starts_with("id,"));
    assert_eq!(resolved[1], json!(1));
    let nested = json!({"content": {"$file": "fixtures/credit-transfer.pain.001.001.09.xml"},
                        "keep": {"$file": "x", "other": 1}});
    let resolved = resolve(&nested);
    assert!(resolved["content"]
        .as_str()
        .expect("text")
        .contains("<NbOfTxs>5</NbOfTxs>"));
    assert_eq!(resolved["keep"], json!({"$file": "x", "other": 1}));
}

#[test]
fn a_transport_failure_fails_the_case() {
    let client = Client::new("http://127.0.0.1:9");
    let case = json!({"id": "x", "method": "GET", "path": "/", "expect": {"status": 200}});
    let failures = run_case(&client, &case);
    assert_eq!(failures.len(), 1);
    assert!(failures[0].starts_with("request failed:"), "{failures:?}");
}

#[test]
fn the_vendored_kit_matches_its_pinned_sums() {
    let sums = fs::read_to_string(kit().join("SHA256SUMS")).expect("read SHA256SUMS");
    let mut listed = Vec::new();
    for line in sums.lines() {
        let (digest, name) = line.split_once("  ").expect("<sha256>  <path>");
        let bytes = fs::read(kit().join(name)).expect("read kit file");
        let actual =
            Sha256::digest(&bytes)
                .iter()
                .fold(String::with_capacity(64), |mut hex, byte| {
                    let _ = write!(hex, "{byte:02x}");
                    hex
                });
        assert_eq!(
            actual, digest,
            "{name} changed: re-vendor the kit from core"
        );
        listed.push(name.to_owned());
    }
    let mut sorted = listed.clone();
    sorted.sort();
    assert_eq!(listed, sorted, "SHA256SUMS is sorted");
    assert_eq!(listed, kit_files(&kit(), ""), "every kit file is listed");
}

fn kit_files(dir: &Path, prefix: &str) -> Vec<String> {
    let mut names = Vec::new();
    for entry in fs::read_dir(dir).expect("read kit dir") {
        let entry = entry.expect("dir entry");
        let name = format!("{prefix}{}", entry.file_name().to_string_lossy());
        if entry.file_type().expect("file type").is_dir() {
            names.extend(kit_files(&entry.path(), &format!("{name}/")));
        } else if name != "SHA256SUMS" {
            names.push(name);
        }
    }
    names.sort();
    names
}
