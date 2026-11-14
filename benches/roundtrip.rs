// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Loopback round trip of every method against a live `pain001 serve` at
//! `PAIN001_BASE_URL` (with `PAIN001_API_KEY` when set): the client's work
//! plus the server's. Skipped, with a message, when the variable is unset.
//!
//! ```sh
//! PAIN001_BASE_URL=http://127.0.0.1:8000 make bench
//! ```

use std::hint::black_box;

use criterion::Criterion;
use pain001_sdk::{Client, MessageType};

const CSV: &str = include_str!("../conformance/fixtures/credit-transfer.pain.001.001.09.csv");
const XML: &str = include_str!("../conformance/fixtures/credit-transfer.pain.001.001.09.xml");
const MT: MessageType = MessageType::Pain00100109;

fn live() -> Option<Client> {
    let Ok(base) = std::env::var("PAIN001_BASE_URL") else {
        eprintln!("skipped: set PAIN001_BASE_URL to run the round-trip benchmarks");
        return None;
    };
    Some(match std::env::var("PAIN001_API_KEY") {
        Ok(key) if !key.is_empty() => Client::new(base).api_key(key),
        _ => Client::new(base),
    })
}

fn reads(c: &mut Criterion, client: &Client) {
    let mut group = c.benchmark_group("roundtrip");
    group.bench_function("health", |b| b.iter(|| client.health().expect("health")));
    group.bench_function("faces", |b| b.iter(|| client.faces().expect("faces")));
    group.bench_function("request", |b| {
        b.iter(|| {
            client
                .request("GET", "/api/v1/health", None)
                .expect("request")
        })
    });
    group.finish();
}

fn files(c: &mut Criterion, client: &Client) {
    let mut group = c.benchmark_group("roundtrip");
    group.bench_function("validate", |b| {
        b.iter(|| {
            client
                .validate(black_box(CSV), "payments.csv", MT, None)
                .expect("validate")
        })
    });
    group.bench_function("generate", |b| {
        b.iter(|| {
            client
                .generate(black_box(CSV), "payments.csv", MT, None)
                .expect("generate")
        })
    });
    group.finish();
}

fn documents(c: &mut Criterion, client: &Client) {
    let mut group = c.benchmark_group("roundtrip");
    group.bench_function("check", |b| {
        b.iter(|| client.check(black_box(XML), None, None).expect("check"))
    });
    group.bench_function("project_face", |b| {
        b.iter(|| {
            client
                .project_face("berlin_group", black_box(XML), None, None)
                .expect("project_face")
        })
    });
    group.finish();
}

fn main() {
    let Some(client) = live() else { return };
    let mut criterion = Criterion::default().configure_from_args();
    reads(&mut criterion, &client);
    files(&mut criterion, &client);
    documents(&mut criterion, &client);
    criterion.final_summary();
}
