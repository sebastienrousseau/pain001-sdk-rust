<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Policies

## Minimum supported Rust version

The floor is Rust 1.85, set as `rust-version` in `Cargo.toml`. It is the
highest `rust-version` among the runtime dependencies (`ureq` 3 and
`ureq-proto` declare 1.85). CI builds and tests on stable and on 1.85.0.

The floor rises only when a dependency update needs it or a language
feature earns it, in its own commit that says why, and the change is
listed in `CHANGELOG.md`.

## Server compatibility

A client at version 0.0.N vendors the conformance kit from core's main
branch at the time of its release, byte for byte, pinned by
`conformance/SHA256SUMS`. CI runs the kit against the pain001 server
pinned in `ci/server-requirements.in`: the latest published release,
currently 0.0.87, while this client prepares 0.0.88.
