// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Every kind of `Error`, provoked against a live server and told apart
//! with one `match`:
//!
//! - `Error::Api` with a string `detail`: a bad IBAN (HTTP 400);
//! - `Error::Api` with an array `detail`, typed by
//!   `Error::request_errors`: an unknown message type (HTTP 422);
//! - `Error::Refused`, a refused generate with its typed findings: a bad
//!   currency (HTTP 422);
//! - `Error::Transport`: nothing listening on the port.
//!
//! `Error::Decode`, a 2xx body of the wrong shape, also covers a generate
//! success without the XML (no document); pain001 0.0.87 never sends one,
//! so it is handled here but not provoked.
//!
//! ```sh
//! PAIN001_BASE_URL=http://127.0.0.1:8000 cargo run --example errors
//! ```
//!
//! Exits non-zero when a call does not fail the way it should.

use pain001_sdk::{Client, Error, MessageType};

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

/// What kind of error `result` holds, printing its details.
fn kind<T>(result: Result<T, Error>) -> &'static str {
    match result {
        Ok(_) => "ok",
        Err(error @ Error::Api { .. }) if error.request_errors().is_some() => {
            for item in error.request_errors().unwrap_or_default() {
                println!("  request error at {:?}: {}", item.loc, item.msg);
            }
            "api-array"
        }
        Err(Error::Api { status, detail }) => {
            println!("  HTTP {status}: {detail}");
            "api"
        }
        Err(Error::Refused { status, response }) => {
            println!("  HTTP {status}: {}", response.message);
            for row in &response.validation_errors {
                println!("  row error: {} {}", row.field, row.message);
            }
            "refused"
        }
        Err(Error::Decode(error)) => {
            println!("  no usable document: {error}");
            "decode"
        }
        Err(Error::Transport(error)) => {
            println!("  transport: {error}");
            "transport"
        }
        Err(other) => {
            println!("  {other}");
            "other"
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = client();
    let mt = MessageType::Pain00100109;
    let bad_iban = fixture("bad-iban.pain.001.001.09.csv")?;
    let bad_currency = fixture("bad-currency.pain.001.001.09.csv")?;
    let credit = fixture("credit-transfer.pain.001.001.09.csv")?;

    let seen = [
        (
            "bad IBAN",
            "api",
            kind(client.validate(&bad_iban, "p.csv", mt, None)),
        ),
        (
            "unknown type",
            "api-array",
            kind(client.validate(&credit, "p.csv", "pain.999.001.01", None)),
        ),
        (
            "bad currency",
            "refused",
            kind(client.generate(&bad_currency, "p.csv", mt, None)),
        ),
        (
            "closed port",
            "transport",
            kind(Client::new("http://127.0.0.1:9").health()),
        ),
    ];
    for (case, want, got) in seen {
        println!("{case}: {got}");
        if want != got {
            return Err(format!("{case}: expected {want}, got {got}").into());
        }
    }
    Ok(())
}
