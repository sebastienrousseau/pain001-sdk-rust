<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# AGENTS.md

`pain001-sdk-rust` is the Rust client of the pain001 suite: a typed,
blocking client (`ureq`, `serde`) for the REST API that `pain001 serve`
exposes. pain001 has one engine, the Python server; each language has a
thin client, and all of them pass one shared conformance kit. This crate
adds no payment logic of its own. Read [CONTRIBUTING.md](CONTRIBUTING.md)
as well.

## Rules

- **Versioning**: `0.0.x`, one 0.0.1 step at a time, in lockstep with
  the pain001 suite. Never bump it as a side effect; the maintainer opens
  a release. When it moves, `Cargo.toml`, `Cargo.lock`, `CITATION.cff`
  and a new `CHANGELOG.md` heading move together.
- **`conformance/` is vendored from core, never edit it here.** Re-vendor
  it byte for byte from the core release of the same version and
  regenerate `conformance/SHA256SUMS`.
- **No payment data in logs.** Request and response bodies are never
  logged or printed; the API key never appears in `Debug` output.
- **No retries by default, and never of a `POST`**: `RetryPolicy` is
  opt-in and retries `GET` calls only (`docs/adr/0004-opt-in-retries.md`).
  No new runtime dependency without a reason in the commit body (licence
  and MSRV checked).
- **`#![forbid(unsafe_code)]`** stays.
- **Complexity**: cyclomatic 10, cognitive 15, 60 lines per function,
  500 lines per file.
- **Commits**: conventional, SSH-signed, `git commit -s`, and an
  `Assisted-by:` trailer for agent work. Never rewrite published history.
- **Licence**: every source file opens with an SPDX
  `Apache-2.0 OR MIT` line (Markdown as an HTML comment on line 1).

## The gate

```sh
make check
PAIN001_BASE_URL=http://127.0.0.1:8000 make conformance
```

`make check` must pass before a push; the conformance run needs a live
`pain001 serve` at the kit's version.

## Leave alone

- `conformance/` (vendored), `LICENSE`, `LICENSE-APACHE`, `LICENSE-MIT`
  and `DCO.txt`.
- The version, except when the maintainer opens a release.
