// SPDX-License-Identifier: Apache-2.0 OR MIT
//! `Client::project_face`: project the kit's pain.001.001.09 into the
//! Berlin Group face and read the loss report, which says what the target
//! shape could not carry.
//!
//! ```sh
//! PAIN001_BASE_URL=http://127.0.0.1:8000 cargo run --example project_face
//! ```
//!
//! Exits non-zero unless the debtor IBAN arrives in the payload.

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
    // The version is read from the namespace when None; bindings fill
    // values the face needs that pain.001 does not hold.
    let projected = client().project_face("berlin_group", &xml, None, None)?;
    let report = &projected.loss_report;
    println!(
        "{} from {}: {} of {} leaves carried, lossless={}",
        projected.face,
        projected.version,
        report.mapped_count,
        report.total_source_elements,
        report.is_lossless
    );
    // An account number is payment data: check it arrived, never print it.
    let carried = projected.payload["debtorAccount"]["iban"].is_string();
    println!("debtorAccount.iban carried: {carried}");
    if !carried {
        return Err("the payload should carry the debtor IBAN".into());
    }
    Ok(())
}
