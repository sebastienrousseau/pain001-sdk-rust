<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Changelog

All notable changes to this project are documented in this file. The
format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and versions advance by 0.0.1 in lockstep with the pain001 suite. The
first release will be 0.0.86.

## [Unreleased]

### Added

- `Client`, a blocking client for the pain001 REST API on `ureq`, with
  `.api_key(..)` (sent as a bearer token) and `.timeout(..)` (30 seconds
  by default).
- One typed method per endpoint: `health`, `validate`, `generate`,
  `check`, `faces` and `project_face`, plus the low-level `request`.
- `MessageType` with the 13 message types of the server's OpenAPI enum;
  methods also accept any string.
- `Error` with `Api { status, detail }`, `Transport` and `Decode`, and
  `Error::request_errors()` for a `422`'s FastAPI `detail` items.
- `RowError` (`field`, `message`, `value`) for row errors. The 0.0.85
  OpenAPI document names FastAPI's request-error schema for these lists
  through a name collision; the type follows what the server sends.
- The conformance kit from pain001 core, vendored in `conformance/` with
  its SHA-256 sums pinned in `conformance/SHA256SUMS`.
- `tests/conformance.rs`, a port of core's reference runner that sends
  every kit case through `request()`, and `tests/typed.rs`, one test per
  method; both run against `PAIN001_BASE_URL` and skip without it.
- `examples/basic.rs`: generate and check a pain.001.001.09 file.
- CI on stable and on Rust 1.85.0, cargo-deny, the conformance job
  against `pain001 serve` 0.0.85 from hash-pinned requirements, the DCO
  check and Markdown lint.
