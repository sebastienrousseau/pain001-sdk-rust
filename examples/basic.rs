// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Generate a pain.001.001.09 file from the conformance kit's synthetic
//! credit-transfer CSV, then check it.
//!
//! ```sh
//! pain001 serve --host 127.0.0.1 --port 8000 &
//! PAIN001_BASE_URL=http://127.0.0.1:8000 cargo run --example basic -- payments.xml
//! ```
//!
//! `PAIN001_API_KEY` is sent as a bearer token when set. The output path
//! defaults to `pain001-example.xml` in the system temporary directory.

use std::path::PathBuf;
use std::process::ExitCode;

use pain001_sdk::{Client, MessageType};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let base_url =
        std::env::var("PAIN001_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:8000".into());
    let mut client = Client::new(base_url);
    if let Ok(key) = std::env::var("PAIN001_API_KEY") {
        client = client.api_key(key);
    }
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("pain001-example.xml"));

    let csv_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/conformance/fixtures/credit-transfer.pain.001.001.09.csv"
    );
    let csv = std::fs::read_to_string(csv_path)?;

    let health = client.health()?;
    println!("server {} is {}", health.version, health.status);

    let generated = client.generate(&csv, "payments.csv", MessageType::Pain00100109, None)?;
    let xml = generated.xml.ok_or(generated.message)?;
    std::fs::write(&output, &xml)?;
    println!("wrote {} ({} bytes)", output.display(), xml.len());

    let report = client.check(&xml, None, None)?;
    println!(
        "check: {} passed={} errors={} warnings={} rungs={}",
        report.message_type,
        report.passed,
        report.errors,
        report.warnings,
        report.rungs.join(",")
    );
    Ok(())
}
