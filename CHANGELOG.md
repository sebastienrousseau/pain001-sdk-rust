<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Changelog

All notable changes to this project are documented in this file. The
format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and versions advance by 0.0.1 in lockstep with the pain001 suite. The
first release will be 0.0.88.

## [Unreleased]

### Changed

- The version moves to 0.0.88 in `Cargo.toml`, both lockfiles,
  `CITATION.cff`, the `SECURITY.md` support table, the bug report form
  and the `KEYS.asc` example. No 0.0.87 was tagged, so 0.0.88 is this
  crate's first release, at the suite's version.
- The test-only `sha2` dependency moves to 0.11.0 (was 0.10.9). The
  kit checksum test hex-encodes the digest itself, since 0.11's output
  type has no `LowerHex`. 0.11.0 is the newest release within the 1.85
  MSRV (0.11.1 needs Rust 1.88), so Dependabot skips 0.11.1 and later
  until the floor rises. The runtime dependencies and the MSRV are
  unchanged.

### Added

- Publishing to crates.io as `pain001-sdk`: `Cargo.toml` now says
  `publish = ["crates-io"]` (was `publish = false`), and
  `.github/workflows/release.yml` releases a `v<version>` tag: version
  check, `cargo package`, a CycloneDX SBOM, `SHA256SUMS` and build
  provenance attestations, a GitHub release titled
  `pain001-sdk-rust <version>` with notes composed by the suite's
  `scripts/release_notes.py` (vendored from core) from
  `releases/v<version>.md`, then `cargo publish` through trusted
  publishing in the `release` environment. A manual run is a dry run,
  and a re-run replaces assets and tolerates a version already
  published. See `docs/adr/0005-publish-to-crates-io.md`.
- `scripts/verify-release-versions` (`make versions`, part of
  `make check` and CI): fails when a lockfile, `CITATION.cff`, the
  `SECURITY.md` or `SUPPORT.md` table, the README install snippet, the
  bug report form or the `KEYS.asc` example disagrees with `Cargo.toml`,
  or, with `--tag`, when the tag names another version.
- `scripts/check-package.sh` (`make package-check`) fails when the crate
  package would ship a demo, fuzz, docs, scripts or CI file, and
  `scripts/api-check.sh` (`make api-check`) runs cargo-semver-checks
  against the newest crates.io release, a no-op until the first one.
- `SUPPORT.md`: where to ask, the supported versions, what to expect.
- `make regression` (`scripts/regression.sh`): installs the pinned
  server into a temporary venv unless `PAIN001_SERVER` names one, starts
  it on a free loopback port with a random API key, runs `make check`,
  `make conformance` (kit, typed tests, every example) and the benchmark
  smoke run, and stops the server and removes the venv even on failure.
- Benchmarks of every public method with criterion 0.7 (a
  dev-dependency; 0.8 needs Rust 1.86, above the floor):
  `benches/client.rs` times the client-side cost with no network,
  encoding each request body and decoding each response from the kit's
  fixtures and the responses in `benches/responses/` (recorded from
  pain001 0.0.87 by `scripts/record-bench-responses.sh`), and
  `benches/roundtrip.rs` times a loopback round trip per method against
  `PAIN001_BASE_URL`, skipped without it. `make bench` runs them,
  `make bench-smoke` runs each once, and CI smoke-runs both on every
  pull request. The client-side bench reaches the encoding and decoding
  through `pain001_sdk::fuzzing`, now also compiled with the new
  `internals` feature, which is not public API.
- One runnable example per method in `examples/`: `health`, `validate`,
  `generate`, `check`, `faces`, `project_face`, `request` (the raw call),
  `errors` (`Error::Api` with a string and an array `detail`,
  `Error::Refused`, `Error::Transport`, and `Error::Decode` for a
  document-less success), `timeout`, `api_key` and `retry`, beside
  `basic`. Each exits non-zero when the server does not answer as it
  expects. `make examples` runs them all, `make conformance` now runs
  them after the kit, and CI's conformance job does so against a server
  started with a throwaway `PAIN001_API_KEY`, which the live tests now
  send when it is set.
- `RetryPolicy`, opt-in through `Client::retry`: `GET` calls (`health`,
  `faces`, `request` with `"GET"`) are tried again after a transport
  error or an HTTP `429` or `503`, up to 3 attempts by default, with
  exponential backoff and jitter (200 ms base, 5 s maximum). A
  `Retry-After` in seconds or as an HTTP date is waited exactly, and one
  longer than the maximum ends the retries. `POST` calls are never
  retried. Without a policy every call makes one attempt, as before.
  See `docs/adr/0004-opt-in-retries.md`.
- Every request sends `User-Agent: pain001-sdk-rust/<version>`, so a
  server's access log shows which client and release made a call; it
  used to be ureq's default. `VERSION` and `USER_AGENT` are exported,
  both derived from `Cargo.toml`.
- A README demo, `.github/demo.gif`, rendered by `make demo` from
  `.github/demo.tape`: `cargo run --example basic` against a
  `pain001 serve` on 127.0.0.1, then the head of the generated file. The
  server comes from `PAIN001_SERVER` or `PATH`; the GIF and tape are
  excluded from the package.
- `Client`, a blocking client for the pain001 REST API on `ureq`, with
  `.api_key(..)` (sent as a bearer token) and `.timeout(..)` (30 seconds
  by default).
- One typed method per endpoint: `health`, `validate`, `generate`,
  `check`, `faces` and `project_face`, plus the low-level `request`.
- `MessageType` with the 13 message types of the server's OpenAPI enum;
  methods also accept any string.
- `Error` with `Api { status, detail }`, `Transport` and `Decode`, and
  `Error::request_errors()` for a `422`'s FastAPI `detail` items.
- `Error::Refused { status, response }` and `Error::refusal()`: pain001
  0.0.87 answers a refused generation (rows that fail validation or a
  scheme rule) with HTTP `422` instead of `200`, and `generate` now
  returns the decoded `GenerateResponse` with its typed
  `validation_errors` and `scheme_violations`, so the findings are not
  lost in a raw `detail`. `Error::status()` still reports `422`, and an
  `Ok` from `generate` always carries the XML.
- `RowError` (`field`, `message`, `value`) for row errors. The 0.0.85
  OpenAPI document names FastAPI's request-error schema for these lists
  through a name collision; the type follows what the server sends.
- The conformance kit from pain001 core, vendored in `conformance/` with
  its SHA-256 sums pinned in `conformance/SHA256SUMS`: kit version 2
  from core 0.0.87, 14 cases, adding the `422` generate refusals (bad
  rows and a UK CHAPS scheme breach) and the bad-LEI and CHAPS purpose
  validation cases with their fixtures.
- `tests/conformance.rs`, a port of core's reference runner that sends
  every kit case through `request()`, and `tests/typed.rs`, one test per
  method; both run against `PAIN001_BASE_URL` and skip without it.
- `examples/basic.rs`: generate and check a pain.001.001.09 file.
- CI on stable and on Rust 1.85.0, cargo-deny, the conformance job
  against `pain001 serve` 0.0.86 from hash-pinned requirements, the DCO
  check and Markdown lint.
- A coverage gate: `make coverage` (part of `make check`) and the CI
  `coverage` job run cargo-llvm-cov and fail below 100% of lines or
  regions in `src/`, the `cfg(fuzzing)` decoding module included. There
  are no exclusions.
- Fuzzing with cargo-fuzz in `fuzz/`, a crate of its own: the
  `response` target decodes any 2xx body as every typed call's
  response, and the `error` target maps any status and body to the
  error a call returns, both checking the documented contracts (only
  `Error::Decode` for a 2xx, the documented `detail` for `Error::Api`,
  `Error::Refused` only for a refusal body). They reach the decoding
  path through `pain001_sdk::fuzzing`, which exists only under
  `cfg(fuzzing)` and is not public API. A pull request runs each target
  for 30 seconds from the seeds in `fuzz/seeds/`.

### Fixed

- `generate` returns `Error::Decode` for a 2xx reply that lacks
  `success: true` or the XML, so an `Ok` always carries the document as
  the README says; it used to return the `GenerateResponse` with
  `xml: None`. A `422` whose body has `success: false` but also carries
  `xml` stays `Error::Api` instead of becoming `Error::Refused`, whose
  response is documented to have no XML. pain001 0.0.87 sends neither
  shape; adding the fuzz targets showed that the client enforced
  neither promise.
