// SPDX-License-Identifier: Apache-2.0 OR MIT
//! `Client::faces`: list the faces a pain.001 can be projected into,
//! with the binding keys each one accepts.
//!
//! ```sh
//! PAIN001_BASE_URL=http://127.0.0.1:8000 cargo run --example faces
//! ```
//!
//! Exits non-zero when the server lists no face.

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
    let faces = client().faces()?;
    for face in &faces {
        println!(
            "{}: {} (bindings: {})",
            face.name,
            face.description,
            face.bindings.join(", ")
        );
    }
    if faces.is_empty() {
        return Err("the server lists no face".into());
    }
    Ok(())
}
