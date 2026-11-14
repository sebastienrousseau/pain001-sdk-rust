// SPDX-License-Identifier: Apache-2.0 OR MIT
//! `Client::api_key` sends `Authorization: Bearer <key>` on every call.
//! A server started with `PAIN001_API_KEY` answers 401 to a call without
//! it. The example validates the kit's CSV without and with the key read
//! from `PAIN001_API_KEY`, and prints what each got.
//!
//! ```sh
//! PAIN001_API_KEY=secret pain001 serve --host 127.0.0.1 --port 8000 &
//! PAIN001_API_KEY=secret PAIN001_BASE_URL=http://127.0.0.1:8000 cargo run --example api_key
//! ```
//!
//! With `PAIN001_API_KEY` set, it exits non-zero unless the keyless call
//! gets 401, the keyed one succeeds and `Debug` output hides the key.

use pain001_sdk::{Client, MessageType};

/// A file from the conformance kit's synthetic fixtures.
fn fixture(name: &str) -> std::io::Result<String> {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/conformance/fixtures/");
    std::fs::read_to_string(format!("{dir}{name}"))
}

/// The status `validate` got: 200 on success, else the error's status.
fn validate(client: &Client, csv: &str) -> Option<u16> {
    match client.validate(csv, "payments.csv", MessageType::Pain00100109, None) {
        Ok(_) => Some(200),
        Err(error) => error.status(),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base = std::env::var("PAIN001_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:8000".into());
    let key = std::env::var("PAIN001_API_KEY").unwrap_or_default();
    let csv = fixture("credit-transfer.pain.001.001.09.csv")?;

    if key.is_empty() {
        println!("PAIN001_API_KEY is unset: the server must be running without one");
    } else {
        let status = validate(&Client::new(&base), &csv);
        println!("without the key: {status:?}");
        if status != Some(401) {
            return Err(format!("a keyless call got {status:?}, want 401").into());
        }
    }
    let keyed = Client::new(&base).api_key(&key);
    // Debug output redacts the key; check that rather than print it.
    let redacted = key.is_empty() || !format!("{keyed:?}").contains(&key);
    println!("Debug output hides the key: {redacted}");
    if !redacted {
        return Err("the client's Debug output showed the API key".into());
    }
    let status = validate(&keyed, &csv);
    println!("with the key: {status:?}");
    if status != Some(200) {
        return Err(format!("the keyed call got {status:?}, want 200").into());
    }
    Ok(())
}
