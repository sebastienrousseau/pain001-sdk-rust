// SPDX-License-Identifier: Apache-2.0 OR MIT
//! `Client::validate`: check a payment file's rows without generating
//! anything. The kit's credit-transfer CSV passes; the same file with one
//! currency broken comes back with a typed row error.
//!
//! ```sh
//! PAIN001_BASE_URL=http://127.0.0.1:8000 cargo run --example validate
//! ```
//!
//! Exits non-zero when either answer is not the expected one.

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
    let client = client();
    let csv = fixture("credit-transfer.pain.001.001.09.csv")?;

    let report = client.validate(&csv, "payments.csv", MessageType::Pain00100109, None)?;
    println!(
        "valid={} rows={} valid_rows={} invalid_rows={}",
        report.is_valid, report.total_rows, report.valid_rows, report.invalid_rows
    );
    if !report.is_valid {
        return Err("the kit's credit transfer should validate".into());
    }

    // A scheme rulebook can be added, comma-separated: Some("sepa-sct").
    let broken = csv.replacen(",EUR,", ",EURO,", 1);
    let report = client.validate(&broken, "payments.csv", MessageType::Pain00100109, None)?;
    for error in &report.errors {
        println!(
            "row error: {} {} (value {:?})",
            error.field, error.message, error.value
        );
    }
    if report.is_valid || report.errors.is_empty() {
        return Err("a bad currency should be reported".into());
    }
    Ok(())
}
