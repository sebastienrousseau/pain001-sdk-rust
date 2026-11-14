// SPDX-License-Identifier: Apache-2.0 OR MIT
//! `Client::generate`: turn a payment file's text into XML, here a
//! pain.008.001.08 direct debit from the kit's synthetic CSV. An `Ok`
//! always carries the document.
//!
//! ```sh
//! PAIN001_BASE_URL=http://127.0.0.1:8000 cargo run --example generate
//! ```
//!
//! Exits non-zero unless the XML holds the file's two transactions.

use pain001_sdk::{Client, MessageType};

/// The client for `PAIN001_BASE_URL`, with `PAIN001_API_KEY` when set.
fn client() -> Client {
    let base = std::env::var("PAIN001_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:8000".into());
    match std::env::var("PAIN001_API_KEY") {
        Ok(key) if !key.is_empty() => Client::new(base).api_key(key),
        _ => Client::new(base),
    }
}

/// A file from the conformance kit's synthetic fixtures.
fn fixture(name: &str) -> std::io::Result<String> {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/conformance/fixtures/");
    std::fs::read_to_string(format!("{dir}{name}"))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let csv = fixture("direct-debit.pain.008.001.08.csv")?;
    let generated = client().generate(&csv, "collections.csv", MessageType::Pain00800108, None)?;
    let xml = generated
        .xml
        .ok_or("an Ok from generate always carries the XML")?;
    println!(
        "{}: {} bytes, suggested name {}",
        generated.message,
        xml.len(),
        generated.filename.as_deref().unwrap_or("(none)")
    );
    if !xml.contains("<NbOfTxs>2</NbOfTxs>") {
        return Err("expected two transactions".into());
    }
    Ok(())
}
