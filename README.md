<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

<p align="center">
  <img src="https://cloudcdn.pro/pain001/v1/logos/pain001.svg" alt="pain001-sdk-rust logo" width="128" />
</p>

<h1 align="center">pain001-sdk-rust</h1>

<p align="center">
  A typed, blocking Rust client for the pain001 REST API, which generates and validates ISO 20022 pain.001 and pain.008 payment files.
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/pain001-sdk-rust/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/pain001-sdk-rust/ci.yml?style=for-the-badge&logo=github" alt="Build" height="28" /></a>
  <a href="#install"><img src="https://img.shields.io/badge/crates.io-not%20yet%20published-lightgrey?style=for-the-badge&color=fc8d62&logo=rust" alt="Registry" height="28" /></a>
  <a href="#documentation"><img src="https://img.shields.io/badge/docs-cargo%20doc-blue?style=for-the-badge&labelColor=555555&logo=docsdotrs" alt="Docs" height="28" /></a>
  <a href="https://scorecard.dev/viewer/?uri=github.com/sebastienrousseau/pain001-sdk-rust"><img src="https://img.shields.io/ossf-scorecard/github.com/sebastienrousseau/pain001-sdk-rust?style=for-the-badge&label=OpenSSF%20Scorecard&logo=openssf" alt="OpenSSF Scorecard" height="28" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0%20OR%20MIT-blue.svg?style=for-the-badge" alt="License: Apache-2.0 OR MIT" height="28" /></a>
  <a href="https://github.com/sebastienrousseau/pain001-sdk-rust/blob/main/docs/POLICIES.md"><img src="https://img.shields.io/badge/Rust-1.85%2B-93450a.svg?style=for-the-badge&logo=rust" alt="Rust 1.85 or newer" height="28" /></a>
</p>

<p align="center">
  No demo GIF: this is a library with no command line of its own, and its output is the pain001 server's. <code>cargo run --example basic</code> shows the same flow in a terminal.
</p>

---

## Contents

**Getting started**

- [Install](#install): a git dependency until the crate is on crates.io
- [Requirements](#requirements): toolchain floor, platforms
- [Quick Start](#quick-start): generate a pain.001 file from a CSV in one call

**The pain001-sdk-rust ecosystem**

- [The pain001-sdk-rust ecosystem](#the-pain001-sdk-rust-ecosystem): the pain001 core, its servers, loaders and SDKs

**Library reference**

- [Capabilities at a glance](#capabilities-at-a-glance): the current surface by theme
- [Ecosystem comparison](#ecosystem-comparison): short matrix
- [Benchmarks](#benchmarks): none published, and why
- [Features](#features): module-level capability list
- [Configuration](#configuration): core options
- [Examples](#examples): runnable example index

**Operational**

- [When not to use pain001-sdk-rust](#when-not-to-use-pain001-sdk-rust): limitations
- [Development](#development): make targets, CI
- [Security](#security): guarantees and compliance
- [Documentation](#documentation): all reference docs
- [Stability guarantees](#stability-guarantees): SemVer axis, output stability, minimum toolchain discipline
- [License](#license)

---

## Install

### As a Rust library

```toml
[dependencies]
pain001-sdk = { git = "https://github.com/sebastienrousseau/pain001-sdk-rust" }
```

The crate is not yet on crates.io (`publish = false` in `Cargo.toml`);
depend on the git repository until it is. It needs a running pain001
server, which is a Python package:

```sh
pip install 'pain001[api]==0.0.86'
pain001 serve --host 127.0.0.1 --port 8000
```

HTTPS uses rustls through the default `tls` feature. A client that only
talks to a loopback server over plain HTTP can drop it with
`default-features = false`.

---

## Requirements

- Rust 1.85 or newer (`rust-version` in `Cargo.toml`), tested in CI on
  stable and on 1.85.0 on Linux ([toolchain policy](docs/POLICIES.md)).
- A pain001 server to talk to. CI runs the conformance kit in
  `conformance/` against pain001 0.0.86, pinned in
  `ci/server-requirements.in`.
- No async runtime: every call blocks the calling thread.

---

## Quick Start

```rust,no_run
use pain001_sdk::{Client, Error, MessageType};
use std::time::Duration;

fn main() -> Result<(), Error> {
    let client = Client::new("http://127.0.0.1:8000").timeout(Duration::from_secs(10));
    let csv = std::fs::read_to_string("payments.csv").expect("read the CSV");

    let generated = client.generate(&csv, "payments.csv", MessageType::Pain00100109, None)?;
    let xml = generated.xml.expect("success carries the XML");

    let report = client.check(&xml, None, None)?;
    println!("{} passed={}", report.message_type, report.passed);
    Ok(())
}
```

`generate` posts the file's text to `POST /api/v1/ui/generate` and
returns the XML inline; an `Ok` always carries it. Rows that fail
validation or a scheme rule come back as `Error::Refused`, a `422` whose
`refusal()` holds the typed `validation_errors` and `scheme_violations`.
A file the server cannot read at all (a bad IBAN, an unknown message
type) comes back as `Error::Api` with the HTTP status and the server's
`detail`, so the caller sees why.

```rust,no_run
# use pain001_sdk::{Client, Error};
# let client = Client::new("http://127.0.0.1:8000");
# let csv = String::new();
match client.generate(&csv, "payments.csv", "pain.001.001.09", Some("uk-chaps")) {
    Ok(generated) => println!("{} bytes", generated.xml.unwrap_or_default().len()),
    Err(Error::Refused { response, .. }) => {
        for row in &response.validation_errors {
            eprintln!("{}: {}", row.field, row.message);
        }
        for violation in &response.scheme_violations {
            eprintln!("{}", violation["rule"]);
        }
    }
    Err(other) => eprintln!("{other}"),
}
```

---

## The pain001-sdk-rust ecosystem

This crate is one of the pain001 suite. pain001 has one engine, the
Python server, and thin typed clients per language that all pass one
shared conformance kit. The suite releases together at one version,
core first; this crate's first release will be 0.0.86.

| Component | Purpose | Use case |
| :--- | :--- | :--- |
| [`pain001`][core] | Generation, validation, CLI and REST | The server this client calls |
| [`pain001-mcp`][mcp] | Model Context Protocol server | Agent tools for payment data |
| [`pain001-lsp`][lsp] | Language Server Protocol server | Editor diagnostics for payment data |
| [`pain001-loader-mt101`][mt101] | SWIFT MT101 loader plugin | Convert supported MT101 inputs |
| [`pain001-loader-xlsx`][xlsx] | Excel loader plugin | Read `.xlsx` and `.xlsm` payment inputs |
| **`pain001-sdk-rust`** | Typed Rust client for the REST API | Call pain001 from a Rust service |

---

## Capabilities at a glance

| Area | Capability | Status |
| :--- | :--- | :--- |
| Health | `health()`: `GET /api/v1/health` | Supported |
| Validation | `validate()`: a CSV, JSON or JSONL file's rows, with optional scheme rulebooks | Supported |
| Generation | `generate()`: pain.001 and pain.008 XML from a file's text | Supported |
| Checking | `check()`: the XSD, MDR, scheme and Schematron ladder on an existing document | Supported |
| Faces | `faces()` and `project_face()`: project a pain.001 into Berlin Group, ISO 2018 names or UK Open Banking, with a loss report | Supported |
| Low level | `request()`: any method and path, returning the status and parsed JSON | Supported |
| Errors | `Error::Api` with status and `detail`; `Transport`; `Decode` | Supported |
| Async jobs | `POST /api/v1/generate` with server-side file paths and job polling | Not wrapped; use `request()` |
| Retries | Automatic retry on failure | Not provided, by design |

---

## Ecosystem comparison

The same REST API can be called with any HTTP client. This crate adds
typed requests and responses, error mapping, and proof against the
suite's conformance kit. Sibling SDKs for Go, Java, .NET and C++ run the
same kit.

| Project | Typed responses | Runs the conformance kit | Async runtime |
| :--- | :---: | :---: | :---: |
| **pain001-sdk-rust** | Yes | Yes, in CI | None (blocking) |
| Raw HTTP with any Rust client | No | No | Your choice |

---

## Benchmarks

None published. Every call's time is the network round trip plus the
pain001 server's own work; the client adds JSON encoding and decoding,
which is not where a payment file's time goes.

| Scenario | Result | Environment |
| :--- | ---: | :--- |
| Not measured | n/a | n/a |

---

## Features

- **`Client`** (`src/client.rs`): `Client::new(base_url)` with
  `.api_key(..)` (sent as `Authorization: Bearer <key>`) and
  `.timeout(..)` (default 30 seconds for the whole call). One method per
  endpoint, plus `request()`. Clones share the connection pool; `Debug`
  output redacts the API key.
- **Types** (`src/types.rs`): `HealthResponse`, `ValidationResponse`,
  `GenerateResponse`, `CheckResponse` with `CheckFinding`, `FaceInfo`,
  `FaceResponse` with `LossReport`, and `RowError` (`field`, `message`,
  `value`) for the row errors in validation and generation results.
  `FaceResponse::payload` stays raw
  `serde_json::Value`, since each face has its own shape. Response
  structs are `#[non_exhaustive]`.
- **`MessageType`**: the 13 message types in the server's OpenAPI enum.
  Methods take `impl AsRef<str>`, so a type a newer server adds works as a
  plain string.
- **`Error`** (`src/error.rs`): `Refused { status, response }` for a
  `422` refusal from `generate`, with the decoded `GenerateResponse`
  (also through `Error::refusal()`); `Api { status, detail }` for any
  other non-2xx response, where `detail` is a string, or an array for a
  `422` that `Error::request_errors()` returns typed as `RequestError`
  items; `Transport` for connection, timeout and TLS failures; `Decode`
  for a 2xx body of the wrong shape. `Error::status()` covers both
  `Refused` and `Api`.

---

## Configuration

| Option | Default | Set with |
| :--- | :--- | :--- |
| Base URL | none | `Client::new("http://127.0.0.1:8000")` |
| API key | none | `.api_key(key)` |
| Timeout for a whole call | 30 seconds | `.timeout(Duration)` |
| HTTPS | on (rustls) | the `tls` Cargo feature |

The client reads no environment variables. The tests and the example read
`PAIN001_BASE_URL`, and the example reads `PAIN001_API_KEY`.

---

## Examples

- [`examples/basic.rs`](examples/basic.rs): generate a pain.001.001.09
  file from the kit's synthetic credit-transfer CSV, write it to disk,
  then check it.

  ```sh
  PAIN001_BASE_URL=http://127.0.0.1:8000 cargo run --example basic -- payments.xml
  ```

CI runs it against a live server on every change.

---

## When not to use pain001-sdk-rust

- **You need async.** The client blocks the calling thread; in an async
  service, call it from a blocking task, or use an async HTTP client
  against the same API.
- **You cannot run a Python server.** The work happens in pain001, so a
  server must be reachable.
- **You want to read files on the server.** The wrapped endpoints take
  the file's text in the request; the path-based `POST /api/v1/generate`
  is reachable only through `request()`.
- **You need retries.** None are made; a payment request that may have
  reached the server is the caller's to retry.

---

## Development

```bash
git clone https://github.com/sebastienrousseau/pain001-sdk-rust
cd pain001-sdk-rust
make check
PAIN001_BASE_URL=http://127.0.0.1:8000 make conformance
```

`make check` runs `cargo fmt --check`, clippy with `-D warnings` and the
complexity lints (`clippy.toml` sets cognitive complexity 15 and 60 lines
per function), the build, the tests, `cargo doc` with warnings denied and
`cargo deny check`. The conformance and typed tests talk to the server at
`PAIN001_BASE_URL` and report themselves skipped when it is unset. CI runs
everything on stable and on 1.85.0, starts `pain001 serve` from a
hash-pinned `ci/server-requirements.txt` for the conformance job, and
checks the DCO and Markdown style. See [`CONTRIBUTING.md`](CONTRIBUTING.md).

---

## Security

The crate has no `unsafe` code (`#![forbid(unsafe_code)]`). It never logs
request or response bodies, which carry payment data, and its `Debug`
output hides the API key. HTTPS uses rustls with the webpki root store.
Use HTTPS and an API key for any server that is not on loopback. Every
dependency is checked by `cargo deny` for RustSec advisories and licences.

Report vulnerabilities according to [`SECURITY.md`](SECURITY.md).

---

## Documentation

API reference: `cargo doc --open` (not yet on docs.rs) ·
[Conformance kit](conformance/README.md) ·
[Developer guide](CONTRIBUTING.md) ·
[Family map](#the-pain001-sdk-rust-ecosystem) ·
[Changelog](CHANGELOG.md) · [Policies](docs/POLICIES.md) ·
[Agent rules](AGENTS.md)

---

## Stability guarantees

The project is pre-1.0 and versions advance in `0.0.1` steps, in lockstep
with the pain001 suite. The breaking axis is the public Rust API: method
signatures, the `Error` variants and the response types. Response structs
and enums are `#[non_exhaustive]`, so a field or message type the server
adds is not breaking. The Rust floor moves only as described in the
[toolchain policy](docs/POLICIES.md).

---

## License

This project is dual-licensed under either:

- Apache License, Version 2.0 ([`LICENSE-APACHE`](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([`LICENSE-MIT`](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option. Dependencies keep their own licences.

[core]: https://github.com/sebastienrousseau/pain001
[mcp]: https://github.com/sebastienrousseau/pain001-mcp
[lsp]: https://github.com/sebastienrousseau/pain001-lsp
[mt101]: https://github.com/sebastienrousseau/pain001-loader-mt101
[xlsx]: https://github.com/sebastienrousseau/pain001-loader-xlsx
