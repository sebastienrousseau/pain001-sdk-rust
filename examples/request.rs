// SPDX-License-Identifier: Apache-2.0 OR MIT
//! `Client::request`: the raw call, for an endpoint the typed methods do
//! not wrap. It returns the status and the parsed JSON whatever the status,
//! so a 404 is an answer here, not an error.
//!
//! ```sh
//! PAIN001_BASE_URL=http://127.0.0.1:8000 cargo run --example request
//! ```
//!
//! Exits non-zero when a status is not the expected one.

use pain001_sdk::Client;
use serde_json::json;

/// The client for `PAIN001_BASE_URL`, with `PAIN001_API_KEY` when set.
fn client() -> Client {
    let base = std::env::var("PAIN001_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:8000".into());
    match std::env::var("PAIN001_API_KEY") {
        Ok(key) if !key.is_empty() => Client::new(base).api_key(key),
        _ => Client::new(base),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = client();

    let (status, body) = client.request("GET", "/api/v1/health", None)?;
    println!("GET /api/v1/health -> {status} {body}");

    let body = json!({"xml": "<Document/>"});
    let (refused, detail) = client.request("POST", "/api/v1/faces/no_such_face", Some(&body))?;
    println!("POST /api/v1/faces/no_such_face -> {refused} {detail}");

    if status != 200 || refused != 404 {
        return Err(format!("expected 200 and 404, got {status} and {refused}").into());
    }
    Ok(())
}
