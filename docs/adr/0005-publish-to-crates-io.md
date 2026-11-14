<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# 5. Publish to crates.io from the release workflow

- Status: accepted
- Date: 2026-11-14

## Context

Until 0.0.88, `Cargo.toml` said `publish = false`, and users depended on
the git repository. The suite's 0.0.88 release publishes each SDK to its
ecosystem's registry; for Rust that is crates.io, as `pain001-sdk` (the
name was free on 2026-11-14 and is not reserved).

`cargo publish` refuses a package whose manifest says `publish = false`.
Two ways round it were considered:

1. keep `publish = false` in the committed manifest and have the release
   workflow rewrite it before publishing;
2. set it in the committed manifest.

The first publishes a manifest that differs from the signed tag: the
package's `Cargo.toml.orig` and `.cargo_vcs_info.json` would record a
dirty tree, `cargo publish` would need `--allow-dirty`, and the crate on
crates.io could not be traced byte for byte to the tagged commit.

## Decision

The committed manifest says `publish = ["crates-io"]`: crates.io only,
never another registry. Only the `publish` job of
`.github/workflows/release.yml` publishes, on a `v<version>` tag, in the
`release` environment, with a short-lived token from crates.io trusted
publishing (`rust-lang/crates-io-auth-action`). No long-lived token is
stored.

crates.io registers a trusted publisher only for a crate that already
exists, so the first version is the exception: the maintainer publishes
0.0.88 once with a scoped token, either by hand from the signed tag or
by setting `CRATES_IO_BOOTSTRAP_TOKEN` in the `release` environment for
that run and deleting it after. Then the trusted publisher is registered
(repository `sebastienrousseau/pain001-sdk-rust`, workflow
`release.yml`, environment `release`) and the token revoked.

The package contents are an allow-list (`include` in `Cargo.toml`),
checked by `scripts/check-package.sh` in CI.

## Consequences

- The published crate matches the signed tag's tree exactly.
- A maintainer could run `cargo publish` locally by mistake; it needs a
  crates.io token on that machine, and the release process in
  `DEVELOPMENT.md` never asks for one after the first release.
- API breakage is checked against the newest crates.io release with
  cargo-semver-checks from the first release on.
