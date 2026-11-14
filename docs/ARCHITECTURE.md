<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Architecture

`pain001-sdk` is a thin, typed, blocking client for the REST API of
`pain001 serve`. Every payment rule, schema and scheme check lives in the
Python server; this crate turns Rust calls into JSON requests and JSON
responses into Rust types and errors. It adds no payment logic of its
own ([ADR 1](adr/0001-thin-client-over-the-server.md)).

## Modules

| Module | File | Responsibility |
| :--- | :--- | :--- |
| `client` | `src/client.rs` | `Client`: configuration (base URL, API key, timeout, retry policy), one method per endpoint, `request`, and the exchange: build the request head, encode the body, send it with ureq, read at most 64 MiB, parse the body |
| `types` | `src/types.rs` | request bodies (crate-private) and the `#[non_exhaustive]` response types, `MessageType` |
| `error` | `src/error.rs` | `Error` and the mapping from a non-2xx status and body to `Api` or `Refused`; the check that a 2xx generate carries the XML |
| `retry` | `src/retry.rs` | `RetryPolicy`: which outcomes are retried, the backoff, `Retry-After` |
| `fuzzing` | `src/fuzzing.rs` | the encoding and decoding without the network; compiled only for fuzzing, the `internals` feature (benches) and tests |
| `stub` | `src/stub.rs` | a loopback HTTP stub for unit tests |

## A call, end to end

```text
Client::generate(content, filename, message_type, scheme)
  -> FilePayload { .. }                    types.rs
  -> exchange: encode as JSON              client.rs (Error::Decode if it cannot)
  -> send: head (method, URL, Accept,      client.rs (Error::Transport for a bad URL)
           User-Agent, Authorization)
  -> once: ureq Agent::run, read body      client.rs (Error::Transport on I/O, timeout, TLS)
     (GET only: retry::run loops here under a RetryPolicy)
  -> parse_body: empty is null, non-JSON is a JSON string
  -> decode: 2xx -> GenerateResponse       client.rs (Error::Decode on a wrong shape)
            non-2xx -> generate_error      error.rs (Refused for a 422 refusal, else Api)
  -> generated: success and xml present    error.rs (Error::Decode otherwise)
```

Every typed method follows this path; `request` stops after
`parse_body` and returns the status and the JSON whatever the status.

## Decisions that shape it

- **Blocking, on ureq.** No async runtime is pulled into the caller's
  program; an async service calls the client from a blocking task
  ([ADR 2](adr/0002-runtime-dependencies.md)).
- **One error type.** `Error` separates what the server said (`Api`,
  `Refused`) from what went wrong on the way (`Transport`) and a reply of
  the wrong shape (`Decode`) ([ADR 3](adr/0003-error-model.md)).
- **No retries unless asked, never of a POST**
  ([ADR 4](adr/0004-opt-in-retries.md)).
- **Published from a signed tag** to crates.io
  ([ADR 5](adr/0005-publish-to-crates-io.md)).

## What keeps it honest

- The conformance kit in `conformance/` is vendored byte for byte from
  pain001 core and pinned by `conformance/SHA256SUMS`; CI runs every case
  against the server version in `ci/server-requirements.txt`.
- `tests/typed.rs` and every example check typed fields against that
  server; the fuzz targets check the decoding contracts on arbitrary
  input; the unit tests cover 100% of lines and regions.
- Request and response bodies carry payment data, so nothing logs them,
  and `Client`'s `Debug` output redacts the API key.
