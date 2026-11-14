// SPDX-License-Identifier: Apache-2.0 OR MIT
//! `Client::health`: `GET /api/v1/health`, which needs no API key.
//!
//! ```sh
//! PAIN001_BASE_URL=http://127.0.0.1:8000 cargo run --example health
//! ```
//!
//! Exits non-zero unless the server reports itself healthy.

use pain001_sdk::Client;

/// The client for `PAIN001_BASE_URL`, with `PAIN001_API_KEY` when set.
fn client() -> Client {
    let base = std::env::var("PAIN001_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:8000".into());
    match std::env::var("PAIN001_API_KEY") {
        Ok(key) if !key.is_empty() => Client::new(base).api_key(key),
        _ => Client::new(base),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let health = client().health()?;
    println!(
        "status={} version={} message={}",
        health.status, health.version, health.message
    );
    if health.status != "healthy" {
        return Err(format!("the server is {}", health.status).into());
    }
    Ok(())
}
