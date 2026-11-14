<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Security Policy

## Supported versions

| Version | Status | Security fixes |
| :--- | :--- | :--- |
| `0.0.88` | Current (first release) | Yes |

0.0.88 is the first release, in lockstep with the pain001 suite. From
then on the latest release receives security fixes; an older release is
fixed by upgrading, since each step is `0.0.1`.

## Reporting a vulnerability

**Do not open a public issue for security reports.**

Use one of the following private channels, in this order of preference:

1. **GitHub Private Vulnerability Reporting**
   <https://github.com/sebastienrousseau/pain001-sdk-rust/security/advisories/new>
   (the maintainers are notified automatically; do this first)
2. **Email**: `security@pain001.com` (PGP available on request)

Please include:

- A description of the vulnerability and its potential impact
- A minimal reproducer (preferably a failing test case)
- The version or commit affected
- Any proposed remediation, if known

These are goals, not guarantees: pain001 is maintained in a personal
capacity, with no SLA and no out-of-hours cover.

**Acknowledgement**: the goal is within 5 working days.
**Triage**: a severity assessment and remediation plan after
acknowledgement.

A vulnerability in the server itself belongs to
[pain001](https://github.com/sebastienrousseau/pain001/security/advisories/new).

## Security posture

- No `unsafe` code: the crate root has `#![forbid(unsafe_code)]`.
- Request and response bodies carry payment data. The client never logs
  them, and its `Debug` output redacts the API key.
- The API key is sent only as an `Authorization: Bearer` header. Use
  HTTPS (rustls, the default `tls` feature) for any server that is not
  on loopback.
- No `POST` is ever retried, so a payment request is never sent twice by
  the client. Retries of `GET` calls are opt-in (`RetryPolicy`).
- `cargo deny check` runs in CI and `make check`: RustSec advisories,
  licences and sources for every dependency, with no ignored advisories.
