// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Client-side cost of every typed method, with no network: encoding the
//! request body the method sends, and decoding the response it reads, from
//! the conformance kit's fixtures and the responses pain001 sent for them
//! (`benches/responses/`, recorded by `scripts/record-bench-responses.sh`).
//!
//! ```sh
//! make bench
//! ```

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use pain001_sdk::fuzzing::{
    check_body, decode, face_body, face_path, file_body, generate, parse_body,
};
use pain001_sdk::{CheckResponse, FaceInfo, FaceResponse, HealthResponse, ValidationResponse};

const CSV: &str = include_str!("../conformance/fixtures/credit-transfer.pain.001.001.09.csv");
const XML: &str = include_str!("../conformance/fixtures/credit-transfer.pain.001.001.09.xml");

const HEALTH: &str = include_str!("responses/health.json");
const VALIDATE: &str = include_str!("responses/validate.json");
const GENERATE: &str = include_str!("responses/generate.json");
const REFUSAL: &str = include_str!("responses/refusal.json");
const CHECK: &str = include_str!("responses/check.json");
const FACES: &str = include_str!("responses/faces.json");
const FACE: &str = include_str!("responses/face.json");

const MT: &str = "pain.001.001.09";

fn encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode");
    group.bench_function("validate", |b| {
        b.iter(|| file_body(black_box(CSV), "payments.csv", MT, None))
    });
    group.bench_function("generate", |b| {
        b.iter(|| file_body(black_box(CSV), "payments.csv", MT, Some("sepa-sct")))
    });
    group.bench_function("check", |b| {
        b.iter(|| check_body(black_box(XML), None, None))
    });
    group.bench_function("project_face", |b| {
        b.iter(|| {
            (
                face_path(black_box("berlin_group")),
                face_body(black_box(XML), None, None),
            )
        })
    });
    group.bench_function("request", |b| {
        let body = serde_json::json!({ "xml": XML });
        b.iter(|| serde_json::to_vec(black_box(&body)))
    });
    group.finish();
}

fn decode_responses(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode");
    group.bench_function("health", |b| {
        b.iter(|| decode::<HealthResponse>(200, black_box(HEALTH)))
    });
    group.bench_function("validate", |b| {
        b.iter(|| decode::<ValidationResponse>(200, black_box(VALIDATE)))
    });
    group.bench_function("generate", |b| {
        b.iter(|| generate(200, black_box(GENERATE)))
    });
    group.bench_function("generate_refused", |b| {
        b.iter(|| generate(422, black_box(REFUSAL)))
    });
    group.bench_function("check", |b| {
        b.iter(|| decode::<CheckResponse>(200, black_box(CHECK)))
    });
    group.bench_function("faces", |b| {
        b.iter(|| decode::<Vec<FaceInfo>>(200, black_box(FACES)))
    });
    group.bench_function("project_face", |b| {
        b.iter(|| decode::<FaceResponse>(200, black_box(FACE)))
    });
    group.bench_function("request", |b| b.iter(|| parse_body(black_box(FACE))));
    group.finish();
}

/// Fails fast if a recorded response no longer decodes, rather than
/// timing an error path.
fn recorded_responses_decode(c: &mut Criterion) {
    decode::<HealthResponse>(200, HEALTH).expect("health.json");
    decode::<ValidationResponse>(200, VALIDATE).expect("validate.json");
    generate(200, GENERATE).expect("generate.json");
    assert!(generate(422, REFUSAL)
        .expect_err("refusal.json")
        .refusal()
        .is_some());
    decode::<CheckResponse>(200, CHECK).expect("check.json");
    decode::<Vec<FaceInfo>>(200, FACES).expect("faces.json");
    decode::<FaceResponse>(200, FACE).expect("face.json");
    encode(c);
    decode_responses(c);
}

criterion_group!(benches, recorded_responses_decode);
criterion_main!(benches);
