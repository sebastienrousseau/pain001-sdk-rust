<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# 2. Four runtime dependencies, blocking on ureq

- Status: accepted
- Date: 2026-11-14 (recording a decision made with the first commit)

## Context

A client needs HTTP (with TLS for remote servers), JSON, and an error
type. Rust's standard library has none of the first two. The choice of
HTTP client decides whether the caller must run an async runtime.

## Decision

Four direct runtime dependencies, each permissively licensed:

| Crate | Why |
| :--- | :--- |
| `ureq` 3 | blocking HTTP/1.1 with no async runtime; rustls for TLS behind the default `tls` feature |
| `serde` | derive the request and response types |
| `serde_json` | JSON encoding and decoding |
| `thiserror` | the `Error` type's `Display` and `source` |

No async runtime, no logging framework, no random-number crate (retry
jitter uses the standard library). `cargo deny` checks advisories,
licences and sources for the whole graph in CI. `ureq` sets the floor:
it and `ureq-proto` declare Rust 1.85.

Dev-only dependencies (`sha2` for the kit checksum test, `criterion` for
the benches) are chosen to stay buildable on the same floor.

## Consequences

- Every call blocks the calling thread; an async service calls it from a
  blocking task, or uses an async HTTP client against the same API.
- A new runtime dependency needs a reason in its commit, with its licence
  and MSRV checked (AGENTS.md).
- Dropping the `tls` feature removes rustls for loopback-only use.
