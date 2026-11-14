<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Packaging

For packagers and anyone vendoring the crate.

## Registry

| Channel | Name | Status |
| :--- | :--- | :--- |
| crates.io | [`pain001-sdk`](https://crates.io/crates/pain001-sdk) | first release 0.0.88, published by the release workflow through trusted publishing ([ADR 5](adr/0005-publish-to-crates-io.md)) |
| docs.rs | [`pain001-sdk`](https://docs.rs/pain001-sdk) | built by docs.rs from each published version |
| GitHub releases | [releases](https://github.com/sebastienrousseau/pain001-sdk-rust/releases) | the `.crate`, a CycloneDX SBOM and `SHA256SUMS`, with build provenance attestations |
| Distribution packages (deb, rpm, Homebrew, Nix, AUR) | none | not applicable: this is a library crate with no binary, and Rust libraries reach distributions through their packaged dependents (Debian, Fedora and others vendor or package crates from crates.io). A distro team that wants it packages the crates.io tarball directly |

## Licence

`Apache-2.0 OR MIT`, at the user's option. Every file is covered by
`REUSE.toml`; `reuse lint` passes in CI. Dependencies keep their own
licences, all permissive (`deny.toml` lists the allowed set).

## Minimum Rust

Rust 1.85 (`rust-version` in `Cargo.toml`), built and tested in CI on
1.85.0 and stable. It rises only when a dependency or a language feature
needs it, in its own commit, listed in the changelog
([policy](POLICIES.md)).

## Dependencies and pins

`Cargo.lock` is committed and every CI build is `--locked`; the published
crate includes it. Runtime dependencies: `ureq`, `serde`, `serde_json`,
`thiserror` ([ADR 2](adr/0002-runtime-dependencies.md)).

## Testing the package offline

The package ships its tests, examples, benches and the conformance kit
they read, so a packager can test the tarball with no network:

```sh
tar -xzf pain001-sdk-0.0.88.crate
cd pain001-sdk-0.0.88
cargo test --locked --offline --all-features
```

The live tests and examples report themselves skipped without
`PAIN001_BASE_URL`. `scripts/check-package.sh` checks in CI that the
package carries no demo, fuzz, docs or CI files.

## Verifying a release

```sh
gh release download v0.0.88 -R sebastienrousseau/pain001-sdk-rust
sha256sum -c SHA256SUMS
gh attestation verify pain001-sdk-0.0.88.crate -R sebastienrousseau/pain001-sdk-rust
git -c gpg.ssh.allowedSignersFile=KEYS.asc verify-tag v0.0.88
```
