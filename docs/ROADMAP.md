<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Roadmap

Direction, not commitments. The suite releases in lockstep, one `0.0.1`
step at a time; this crate follows core's method surface.

## 0.0.89 (this release)

- In lockstep with pain001 0.0.89: CI runs the conformance kit against
  it, and cargo-semver-checks compares the API with 0.0.88 from
  crates.io on every pull request.

## 0.0.88

- First release on crates.io, with the API reference on docs.rs.
- Opt-in `RetryPolicy` for `GET` calls.
- An example and a benchmark for every method; 100% line and region
  coverage, complexity ceilings, REUSE and the rendered manual enforced
  in CI.

## Next

- Wrap the asynchronous job endpoints (`POST /api/v1/generate` with
  server-side paths and job polling), today reachable through `request`,
  when core declares them stable for clients.
- Follow every change to core's conformance kit in the same suite
  release.

## Not planned

- An async client: the crate stays runtime-free; an async caller uses a
  blocking task ([ADR 2](adr/0002-runtime-dependencies.md)).
- Payment logic in Rust: the server is the engine
  ([ADR 1](adr/0001-thin-client-over-the-server.md)).
- Retrying a `POST` ([ADR 4](adr/0004-opt-in-retries.md)).
