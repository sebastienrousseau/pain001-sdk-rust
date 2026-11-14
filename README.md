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
  <a href="https://crates.io/crates/pain001-sdk"><img src="https://img.shields.io/crates/v/pain001-sdk.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Registry" height="28" /></a>
  <a href="https://docs.rs/pain001-sdk"><img src="https://img.shields.io/docsrs/pain001-sdk?style=for-the-badge&labelColor=555555&logo=docsdotrs" alt="Docs" height="28" /></a>
  <a href="https://scorecard.dev/viewer/?uri=github.com/sebastienrousseau/pain001-sdk-rust"><img src="https://img.shields.io/ossf-scorecard/github.com/sebastienrousseau/pain001-sdk-rust?style=for-the-badge&label=OpenSSF%20Scorecard&logo=openssf" alt="OpenSSF Scorecard" height="28" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0%20OR%20MIT-blue.svg?style=for-the-badge" alt="License: Apache-2.0 OR MIT" height="28" /></a>
  <a href="https://github.com/sebastienrousseau/pain001-sdk-rust/blob/main/docs/POLICIES.md"><img src="https://img.shields.io/badge/Rust-1.85%2B-93450a.svg?style=for-the-badge&logo=rust" alt="Rust 1.85 or newer" height="28" /></a>
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/sebastienrousseau/pain001-sdk-rust/main/.github/demo.gif" alt="cargo run --example basic against a local pain001 server: the SDK reads the server health, generates a pain.001.001.09 file from the conformance kit's synthetic CSV, checks it, and the file's first lines are shown" width="100%" />
</p>

---

## Contents

**Getting started**

- [Install](#install): from crates.io, or the git repository
- [Requirements](#requirements): toolchain floor, platforms
- [Quick Start](#quick-start): generate a pain.001 file from a CSV in one call

**The pain001-sdk-rust ecosystem**

- [The pain001-sdk-rust ecosystem](#the-pain001-sdk-rust-ecosystem): the pain001 core, its servers, loaders, tools and SDKs

**Library reference**

- [Capabilities at a glance](#capabilities-at-a-glance): the current surface by theme
- [Ecosystem comparison](#ecosystem-comparison): short matrix; full table at [`docs/COMPARISON.md`](docs/COMPARISON.md)
- [Benchmarks](#benchmarks): headline numbers; full table at [`docs/BENCHMARKS.md`](docs/BENCHMARKS.md)
- [Features](#features): module-level capability list
- [Configuration](#configuration): core options
- [Examples](#examples): runnable example index

**Operational**

- [When not to use pain001-sdk-rust](#when-not-to-use-pain001-sdk-rust): limitations
- [Development](#development): make targets, fuzzing, CI
- [Security](#security): guarantees and compliance
- [Documentation](#documentation): all reference docs
- [Stability guarantees](#stability-guarantees): SemVer axis, output stability, minimum toolchain discipline
- [License](#license)

---

## Install

### As a Rust library

```toml
[dependencies]
pain001-sdk = "0.0.88"
```

0.0.88 is the first release published to crates.io. Before it is out,
depend on the repository instead:
`pain001-sdk = { git = "https://github.com/sebastienrousseau/pain001-sdk-rust" }`.
The client needs a running pain001 server, which is a Python package:

```sh
pip install 'pain001[api]==0.0.87'
pain001 serve --host 127.0.0.1 --port 8000
```

HTTPS uses rustls through the default `tls` feature. A client that only
talks to a loopback server over plain HTTP can drop it with
`default-features = false`. `cargo add pain001-sdk` does the same as the
snippet above.

---

## Requirements

- Rust 1.85 or newer (`rust-version` in `Cargo.toml`), built and tested
  in CI on 1.85.0 and stable on Linux ([toolchain policy](docs/POLICIES.md)).
- A pain001 server to talk to. CI runs the conformance kit in
  `conformance/` against pain001 0.0.87, the latest release, pinned in
  `ci/server-requirements.in`.
- No async runtime: every call blocks the calling thread. Any platform
  ureq and rustls support; CI runs on Linux, development happens on macOS.

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
shared conformance kit. The suite releases together at one version, core
first; this crate's first release is 0.0.88.

| Component | Purpose | Use case |
| :--- | :--- | :--- |
| [`pain001`][core] | Generation, validation, CLI and REST | The server this client calls |
| [`pain001-mcp`][mcp] | Model Context Protocol server | Agent tools for payment data |
| [`pain001-lsp`][lsp] | Language Server Protocol server | Editor diagnostics for payment data |
| [`pain001-loader-mt101`][mt101] | SWIFT MT101 loader plugin | Convert supported MT101 inputs |
| [`pain001-loader-xlsx`][xlsx] | Excel loader plugin | Read `.xlsx` and `.xlsm` payment inputs |
| [`pain001-mockbank`][mockbank] | Synthetic SFTP bank-response test service | Generate pain.002 replies in integration tests |
| [`pain001-plugin-template`][template] | Cookiecutter plugin scaffold | Develop an external loader or validator |
| [`pain001-fast`][fast] | Experimental native validation extension | Optional IBAN, BIC and character-set helpers |
| [`pain001.com`][site] | Website and browser demo | Browse documentation and try the library |
| [`pain001-sdk-go`][go], [`-java`][java], [`-dotnet`][dotnet], [`-cpp`][cpp] | Typed clients in other languages | Call pain001 from Go, Java, .NET or C++ |
| **`pain001-sdk-rust`** | Typed Rust client for the REST API | Call pain001 from a Rust service |

---

## Capabilities at a glance

| Area | Capability | Status |
| :--- | :--- | :--- |
| Health | `health()`: `GET /api/v1/health` | Supported |
| Validation | `validate()`: a CSV, JSON or JSONL file's rows, with optional scheme rulebooks | Supported |
| Generation | `generate()`: pain.001 and pain.008 XML from a file's text; a refusal is a typed `Error::Refused` | Supported |
| Checking | `check()`: the XSD, MDR, scheme and Schematron ladder on an existing document | Supported |
| Faces | `faces()` and `project_face()`: project a pain.001 into Berlin Group, ISO 2018 names or UK Open Banking, with a loss report | Supported |
| Low level | `request()`: any method and path, returning the status and parsed JSON | Supported |
| Errors | `Error::Api`, `Refused`, `Transport`, `Decode` | Supported |
| Retries | `RetryPolicy`: opt-in retries of `GET` calls on transport errors, 429 and 503, honouring `Retry-After` | Supported, off by default |
| Async jobs | `POST /api/v1/generate` with server-side file paths and job polling | Not wrapped; use `request()` |

---

## Ecosystem comparison

The suite's five SDKs share one method surface and run core's conformance kit against a live `pain001 serve` in CI. Core's CI also generates a Python client from the OpenAPI document, which it imports but does not run against the kit.

| Project | Language (minimum) | Conformance kit in CI | Types | Runtime dependencies | Call model |
| :--- | :---: | :---: | :---: | :--- | :--- |
| pain001-sdk-go | Go 1.26 | Yes | Hand-written | None (standard library) | Blocking, `context.Context` |
| pain001-sdk-java | Java 17 | Yes | Hand-written | Jackson databind | Blocking |
| **pain001-sdk-rust** | **Rust 1.85** | **Yes** | **Hand-written (serde)** | **ureq, serde, serde_json, thiserror** | **Blocking** |
| pain001-sdk-dotnet | .NET 10 | Yes | Hand-written | None | async/await, `CancellationToken` |
| pain001-sdk-cpp | C++17 | Yes | Hand-written | libcurl, nlohmann/json | Blocking |
| openapi-generator client (core CI artifact) | Python | No, CI imports it only | Generated | Generator's | Blocking |
| Raw HTTP | Any | No | None | Your HTTP client | Yours |

See [`docs/COMPARISON.md`](docs/COMPARISON.md) for the evidence and complete matrix.

---

## Benchmarks

Criterion benchmarks cover every public method: the client-side cost
with no network (encoding each request body, decoding each response) and
a loopback round trip against `pain001 serve`. The client's work is
microseconds; the server's is milliseconds. Medians from the first of two
runs on a busy laptop (load average 25 to 63), so read them as orders of
magnitude:

| Scenario | Result | Environment |
| :--- | ---: | :--- |
| Decode a `generate` response (4,466 bytes, XML inside) | 12.0 µs | Apple A18 Pro, macOS 27.2, rustc 1.99.0 |
| Encode a `check` request (the kit's pain.001) | 10.0 µs | same |
| Round trip, `health` | 0.44 ms | same, pain001 0.0.87 on loopback |
| Round trip, `generate` (5 rows) | 16.4 ms | same |
| Round trip, `project_face` | 24.2 ms | same |

See [`docs/BENCHMARKS.md`](docs/BENCHMARKS.md) for methodology and full results.

---

## Features

- **`Client`** (`src/client.rs`): `Client::new(base_url)` with
  `.api_key(..)` (sent as `Authorization: Bearer <key>`), `.timeout(..)`
  (default 30 seconds per attempt) and `.retry(RetryPolicy)`. One method
  per endpoint, plus `request()`. Every request sends
  `User-Agent: pain001-sdk-rust/<version>`. Clones share the connection
  pool; `Debug` output redacts the API key.
- **Types** (`src/types.rs`): `HealthResponse`, `ValidationResponse`,
  `GenerateResponse`, `CheckResponse` with `CheckFinding`, `FaceInfo`,
  `FaceResponse` with `LossReport`, `RowError` and `RequestError`.
  `FaceResponse::payload` stays raw `serde_json::Value`, since each face
  has its own shape. Response structs are `#[non_exhaustive]`.
- **`MessageType`**: the 13 message types in the server's OpenAPI enum.
  Methods take `impl AsRef<str>`, so a type a newer server adds works as a
  plain string.
- **`Error`** (`src/error.rs`): `Refused { status, response }` for a
  `422` refusal from `generate`, with the decoded `GenerateResponse`
  (also through `Error::refusal()`); `Api { status, detail }` for any
  other non-2xx response, typed by `Error::request_errors()` for a `422`
  `detail` array; `Transport` for connection, timeout and TLS failures;
  `Decode` for a 2xx body of the wrong shape. `Error::status()` covers
  `Refused` and `Api`.
- **`RetryPolicy`** (`src/retry.rs`): 3 attempts, 200 ms base and 5 s
  maximum delay by default, for `GET` calls only
  ([ADR 4](docs/adr/0004-opt-in-retries.md)).

---

## Configuration

| Option | Default | Set with |
| :--- | :--- | :--- |
| Base URL | none | `Client::new("http://127.0.0.1:8000")` |
| API key | none | `.api_key(key)` |
| Timeout per attempt | 30 seconds (`DEFAULT_TIMEOUT`) | `.timeout(Duration)` |
| Retries | none | `.retry(RetryPolicy::new())`, tuned with `with_max_attempts`, `with_base_delay`, `with_max_delay` |
| HTTPS | on (rustls) | the `tls` Cargo feature |

The client reads no environment variables. The tests, examples and
benchmarks read `PAIN001_BASE_URL` and `PAIN001_API_KEY`.

---

## Examples

One program per method, each run against a live server in CI's
conformance job and exiting non-zero when an answer is wrong:

```sh
PAIN001_BASE_URL=http://127.0.0.1:8000 cargo run --example health
PAIN001_BASE_URL=http://127.0.0.1:8000 make examples   # all of them
```

| Example | Shows |
| :--- | :--- |
| [`basic`](examples/basic.rs) | health, generate a pain.001.001.09 file, write it, check it |
| [`health`](examples/health.rs) | `health()` |
| [`validate`](examples/validate.rs) | `validate()`, a valid file and a typed row error |
| [`generate`](examples/generate.rs) | `generate()` of a pain.008.001.08 direct debit |
| [`check`](examples/check.rs) | `check()`, the rungs and findings |
| [`faces`](examples/faces.rs) | `faces()` and their bindings |
| [`project_face`](examples/project_face.rs) | `project_face()` into Berlin Group, with the loss report |
| [`request`](examples/request.rs) | `request()`, the raw call, with a 200 and a 404 |
| [`errors`](examples/errors.rs) | `Error::Api` (string and array `detail`), `Refused`, `Transport`, and `Decode` for a document-less success |
| [`timeout`](examples/timeout.rs) | `.timeout()` firing against a silent listener |
| [`api_key`](examples/api_key.rs) | `.api_key()`: 401 without it, 200 with it |
| [`retry`](examples/retry.rs) | `RetryPolicy`: a `GET` retried past a 503, a `POST` not |

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
- **You need a `POST` retried.** The client never repeats one, since that
  could submit a payment file twice; that decision stays with the caller.

---

## Development

```bash
git clone https://github.com/sebastienrousseau/pain001-sdk-rust
cd pain001-sdk-rust
make check
make regression
```

`make check` runs rustfmt, clippy with `-D warnings` and the complexity
lints, the build (benches included), the tests at 100% line and region
coverage (cargo-llvm-cov), rustdoc with warnings denied, `cargo deny`,
the complexity ceilings (cyclomatic 10, cognitive 15, Halstead difficulty
30, 60 lines per function), the version check and the package contents
check. `make regression` installs the pinned server into a temporary
venv, starts it on loopback with an API key, and adds the conformance
kit, every example and the benchmark smoke run. The fuzz targets in
`fuzz/` run for 30 seconds each on every pull request. See
[`DEVELOPMENT.md`](DEVELOPMENT.md) for every gate and its CI job.

---

## Security

The crate has no `unsafe` code (`#![forbid(unsafe_code)]`). It never logs
request or response bodies, which carry payment data, and its `Debug`
output hides the API key. HTTPS uses rustls with the webpki root store;
use HTTPS and an API key for any server that is not on loopback. A
response body is read up to 64 MiB. No `POST` is ever retried. The
response decoding and error mapping are fuzzed with cargo-fuzz on every
pull request from committed seeds, and every dependency is checked by
`cargo deny` for RustSec advisories and licences. CodeQL and OpenSSF
Scorecard run on `main`; releases carry an SBOM and build provenance.

Report vulnerabilities according to [`SECURITY.md`](SECURITY.md).

---

## Documentation

[User manual](https://sebastienrousseau.github.io/pain001-sdk-rust/) ·
[API reference (docs.rs)](https://docs.rs/pain001-sdk) ·
[Developer docs](DEVELOPMENT.md) ·
[Ecosystem map](#the-pain001-sdk-rust-ecosystem)

Also: [Architecture](docs/ARCHITECTURE.md) ·
[Decision records](docs/adr/README.md) ·
[Conformance kit](conformance/README.md) ·
[Changelog](CHANGELOG.md) · [Policies](docs/POLICIES.md) ·
[Packaging](docs/packaging.md) · [Roadmap](docs/ROADMAP.md) ·
[Support](SUPPORT.md) · [Governance](GOVERNANCE.md) ·
[Agent rules](AGENTS.md)

---

## Stability guarantees

The project is pre-1.0 and versions advance in `0.0.1` steps, in lockstep
with the pain001 suite. The breaking axis is the public Rust API: method
signatures, the `Error` variants, `RetryPolicy` and the response types;
cargo-semver-checks guards it against the last crates.io release from
0.0.88 on. Response structs and enums are `#[non_exhaustive]`, so a field
or message type the server adds is not breaking. The client adds no
output of its own: what a call returns is what the server sent, decoded,
so behaviour changes follow the server's release notes and the
conformance kit. The `internals` feature and `pain001_sdk::fuzzing` are
not public API. The Rust floor moves only as described in the
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
[mockbank]: https://github.com/sebastienrousseau/pain001-mockbank
[template]: https://github.com/sebastienrousseau/pain001-plugin-template
[fast]: https://github.com/sebastienrousseau/pain001-fast
[site]: https://pain001.com
[go]: https://github.com/sebastienrousseau/pain001-sdk-go
[java]: https://github.com/sebastienrousseau/pain001-sdk-java
[dotnet]: https://github.com/sebastienrousseau/pain001-sdk-dotnet
[cpp]: https://github.com/sebastienrousseau/pain001-sdk-cpp
