<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Getting support

## Read first

- [README.md](https://github.com/sebastienrousseau/pain001-sdk-rust/blob/main/README.md): install, quick start, configuration and the
  example index.
- [The user manual](https://sebastienrousseau.github.io/pain001-sdk-rust/),
  built from [`docs/`](https://github.com/sebastienrousseau/pain001-sdk-rust/tree/main/docs), and the API reference on
  [docs.rs](https://docs.rs/pain001-sdk).
- [`examples/`](https://github.com/sebastienrousseau/pain001-sdk-rust/tree/main/examples): one runnable program per method, each run
  against a live `pain001 serve` in CI.
- [DEVELOPMENT.md](https://github.com/sebastienrousseau/pain001-sdk-rust/blob/main/DEVELOPMENT.md): building, testing and every CI gate.

## Supported versions

| Version | Status | Supported |
| :--- | :--- | :--- |
| `0.0.88` | Current (first release) | Yes |

The latest release is supported. Versions advance by `0.0.1` in lockstep
with the pain001 suite, so upgrading is the fix for an older one.

## Where to ask

- **A bug in this crate**: open an issue with the
  [bug report form](https://github.com/sebastienrousseau/pain001-sdk-rust/issues/new/choose).
  Include the crate version, the server's `GET /api/v1/health` version
  and `rustc --version`. Never paste real payment data.
- **A feature request**: the feature request form on the same page.
- **A problem in the server** (what it generates, validates or refuses):
  [pain001 issues](https://github.com/sebastienrousseau/pain001/issues).
- **A security problem**: never a public issue; follow
  [SECURITY.md](https://github.com/sebastienrousseau/pain001-sdk-rust/blob/main/SECURITY.md).

## What to expect

pain001 is maintained by one person in a personal capacity. There is no
SLA and no out-of-hours cover; issues are triaged as time allows, and
security reports first (see [GOVERNANCE.md](https://github.com/sebastienrousseau/pain001-sdk-rust/blob/main/GOVERNANCE.md)).
