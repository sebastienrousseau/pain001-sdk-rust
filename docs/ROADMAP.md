<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Roadmap

Direction, not commitments. The suite releases in lockstep, one `0.0.1`
step at a time; this crate follows core's method surface.

## 0.0.88 (this release)

- First release on crates.io, with the API reference on docs.rs.
- Opt-in `RetryPolicy` for `GET` calls.
- An example and a benchmark for every method; 100% line and region
  coverage, complexity ceilings, REUSE and the rendered manual enforced
  in CI.

## Next

- Run cargo-semver-checks against 0.0.88 on every pull request (wired,
  waiting for the first published version).
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
