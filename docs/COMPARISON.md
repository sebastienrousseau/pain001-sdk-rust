<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Ecosystem comparison

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

## Evidence

Each row's claims can be checked in that project's manifest and in the
CI job that runs the kit:

| Project | Manifest (language floor, dependencies) | Conformance job |
| :--- | :--- | :--- |
| pain001-sdk-go | [`go.mod`](https://github.com/sebastienrousseau/pain001-sdk-go/blob/main/go.mod) | [`ci.yml`, job `conformance`](https://github.com/sebastienrousseau/pain001-sdk-go/blob/main/.github/workflows/ci.yml) |
| pain001-sdk-java | [`pom.xml`](https://github.com/sebastienrousseau/pain001-sdk-java/blob/main/pom.xml) | [`ci.yml`, job `conformance`](https://github.com/sebastienrousseau/pain001-sdk-java/blob/main/.github/workflows/ci.yml) |
| pain001-sdk-rust | [`Cargo.toml`](https://github.com/sebastienrousseau/pain001-sdk-rust/blob/main/Cargo.toml) (`rust-version`, `[dependencies]`) | [`ci.yml`, job `conformance`](https://github.com/sebastienrousseau/pain001-sdk-rust/blob/main/.github/workflows/ci.yml) |
| pain001-sdk-dotnet | [`src/Pain001.Sdk/Pain001.Sdk.csproj`](https://github.com/sebastienrousseau/pain001-sdk-dotnet/blob/main/src/Pain001.Sdk/Pain001.Sdk.csproj) | [`ci.yml`, job `conformance`](https://github.com/sebastienrousseau/pain001-sdk-dotnet/blob/main/.github/workflows/ci.yml) |
| pain001-sdk-cpp | [`CMakeLists.txt`](https://github.com/sebastienrousseau/pain001-sdk-cpp/blob/main/CMakeLists.txt) | [`ci.yml`, job `conformance`](https://github.com/sebastienrousseau/pain001-sdk-cpp/blob/main/.github/workflows/ci.yml) |
| openapi-generator client | [core's `sdk.yml`](https://github.com/sebastienrousseau/pain001/blob/main/.github/workflows/sdk.yml) | none: the generated client is imported, not run against the kit |

## This crate's row

- **Rust 1.85**: `rust-version` in `Cargo.toml`; CI builds and tests
  every target on 1.85.0 and stable.
- **Conformance kit**: `tests/conformance.rs` runs all 14 cases of kit
  version 2 through `Client::request`; `tests/typed.rs` and the twelve
  examples check typed fields against the same server.
- **Types**: hand-written structs with serde derives in `src/types.rs`,
  checked against `conformance/openapi.json` (the `MessageType` enum is
  compared in a unit test).
- **Runtime dependencies**: the four direct ones; rustls comes through
  ureq's default `tls` feature and can be dropped
  ([ADR 2](adr/0002-runtime-dependencies.md)).
- **Blocking**: no async runtime; every call blocks the calling thread.

## When raw HTTP is enough

Any HTTP client can call the API. This crate adds typed requests and
responses, one error type that separates a refusal from a transport
failure, opt-in retries for `GET` calls, and the conformance kit run on
every change.
