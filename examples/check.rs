// SPDX-License-Identifier: Apache-2.0 OR MIT
//! `Client::check`: run the validation ladder (XSD, MDR, scheme,
//! Schematron) on an existing document, here the kit's pain.001.001.09.
//!
//! ```sh
//! PAIN001_BASE_URL=http://127.0.0.1:8000 cargo run --example check
//! ```
//!
//! Exits non-zero unless the document passes.

use pain001_sdk::Client;

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
    let xml = fixture("credit-transfer.pain.001.001.09.xml")?;
    // Some("sepa-sct") adds a scheme profile, Some("sepa") a Schematron preset.
    let report = client().check(&xml, None, None)?;
    println!(
        "{} passed={} errors={} warnings={} rungs={}",
        report.message_type,
        report.passed,
        report.errors,
        report.warnings,
        report.rungs.join(",")
    );
    for finding in &report.findings {
        println!(
            "  {} {} {}: {}",
            finding.rung, finding.severity, finding.rule, finding.message
        );
    }
    if !report.passed {
        return Err("the kit's document should pass".into());
    }
    Ok(())
}
