// SPDX-License-Identifier: Apache-2.0 OR MIT
//! `Client::retry` with a `RetryPolicy`: `GET` calls are tried again
//! after a transport error or a 429 or 503. A loopback stand-in answers
//! 503 with `Retry-After: 0` once, then forwards the call to the real server, so
//! the retry is visible; a `POST` is never retried.
//!
//! ```sh
//! PAIN001_BASE_URL=http://127.0.0.1:8000 cargo run --example retry
//! ```
//!
//! Exits non-zero unless the `GET` succeeds on its second attempt and the
//! `POST` gets the 503 on its only one.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use pain001_sdk::{Client, RetryPolicy};

const UNAVAILABLE: &[u8] = b"HTTP/1.1 503 Service Unavailable\r\nRetry-After: 0\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}";

/// Answers the first two connections with 503, then forwards the third
/// one's request line to the real server at `upstream` with a client of
/// its own, and relays the status and body back.
fn stand_in(listener: TcpListener, upstream: Client) {
    for (n, stream) in listener.incoming().take(3).enumerate() {
        let Ok(mut stream) = stream else { return };
        let mut request = [0u8; 4096];
        let read = stream.read(&mut request).unwrap_or(0);
        if n < 2 {
            let _ = stream.write_all(UNAVAILABLE);
            continue;
        }
        let head = String::from_utf8_lossy(&request[..read]);
        let path = head.split(' ').nth(1).unwrap_or("/");
        let (status, body) = upstream
            .request("GET", path, None)
            .unwrap_or((502, serde_json::Value::Null));
        let body = body.to_string();
        let answer = format!(
            "HTTP/1.1 {status} Relayed\r\nContent-Type: application/json\r\n\
             Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let _ = stream.write_all(answer.as_bytes());
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base = std::env::var("PAIN001_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:8000".into());
    // Health needs no API key, so neither client sends one.
    let upstream = Client::new(base);
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let url = format!("http://{}", listener.local_addr()?);
    thread::spawn(move || stand_in(listener, upstream));

    let policy = RetryPolicy::new().with_base_delay(Duration::from_millis(50));
    let client = Client::new(url).retry(policy);

    // POST: one attempt, so the stand-in's first 503 is the answer.
    let (status, _) = client.request("POST", "/api/v1/health", None)?;
    println!("POST, never retried: {status}");
    // GET: the second 503 is retried; the third attempt reaches the server.
    let health = client.health()?;
    println!("GET, retried once: {} ({})", health.status, health.version);
    if status != 503 {
        return Err(format!("the POST got {status}, want 503").into());
    }
    Ok(())
}
