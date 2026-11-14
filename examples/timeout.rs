// SPDX-License-Identifier: Apache-2.0 OR MIT
//! `Client::timeout` bounds a whole call (connect, send and read); the
//! default is `DEFAULT_TIMEOUT`, 30 seconds. A loopback listener that
//! accepts and never answers shows the timeout firing as
//! `Error::Transport`; the same short timeout is then plenty for the real
//! server's health call.
//!
//! ```sh
//! PAIN001_BASE_URL=http://127.0.0.1:8000 cargo run --example timeout
//! ```
//!
//! Exits non-zero unless the silent call times out and the real one does
//! not.

use std::net::TcpListener;
use std::time::{Duration, Instant};

use pain001_sdk::{Client, Error, DEFAULT_TIMEOUT};

/// The client for `PAIN001_BASE_URL`, with `PAIN001_API_KEY` when set.
fn client() -> Client {
    let base = std::env::var("PAIN001_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:8000".into());
    match std::env::var("PAIN001_API_KEY") {
        Ok(key) if !key.is_empty() => Client::new(base).api_key(key),
        _ => Client::new(base),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let limit = Duration::from_millis(500);
    println!("default timeout: {DEFAULT_TIMEOUT:?}, this example: {limit:?}");

    // Accepts the connection (the kernel queues it) but never answers.
    let silent = TcpListener::bind("127.0.0.1:0")?;
    let started = Instant::now();
    let result = Client::new(format!("http://{}", silent.local_addr()?))
        .timeout(limit)
        .health();
    println!(
        "silent server: {:?} after {:?}",
        result.as_ref().err(),
        started.elapsed()
    );
    if !matches!(result, Err(Error::Transport(_))) {
        return Err("the silent server should time out".into());
    }

    let health = client().timeout(limit).health()?;
    println!("real server: {} within {limit:?}", health.status);
    Ok(())
}
