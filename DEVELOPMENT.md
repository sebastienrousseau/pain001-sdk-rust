<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Development

The single entry point for working on `pain001-sdk-rust`: the toolchain,
every CI gate and how to run it locally, the test layout, and how a
release is made. Links point at the repository, so they work both on
GitHub and in the [user manual](https://sebastienrousseau.github.io/pain001-sdk-rust/).

## Toolchain

| Tool | Version | Used by |
| :--- | :--- | :--- |
| Rust | stable, and 1.85.0 (the floor, `rust-version`) | everything |
| clippy, rustfmt, llvm-tools-preview | with the toolchain | `make lint`, `make fmt-check`, `make coverage` |
| cargo-deny | 0.20.2 | `make deny` |
| cargo-llvm-cov | 0.9.1 | `make coverage` |
| rust-code-analysis-cli | 0.0.25 | `make complexity` |
| cargo-semver-checks | 0.51.0 | `make api-check` (once a version is on crates.io) |
| cargo-fuzz and a nightly toolchain | 0.13.2 | the fuzz targets |
| Python | 3.12 | the pinned server, `scripts/*.py` |
| uv (or `python3 -m venv`) | any | installing the server |
| Node.js (for `npx`), codespell, reuse | 22, 2.4.x, 6.x | `make docs-lint` |

```sh
rustup toolchain install stable 1.85.0 --component clippy,rustfmt,llvm-tools-preview
cargo install --locked cargo-deny@0.20.2 cargo-llvm-cov@0.9.1 rust-code-analysis-cli@0.0.25
```

The [devcontainer](https://github.com/sebastienrousseau/pain001-sdk-rust/tree/main/.devcontainer)
has all of these at the same versions. `pre-commit install` runs the fast
checks on every commit (`.pre-commit-config.yaml`, local hooks only).

## The gates

`make check` runs what CI's static jobs run; `make regression` runs
everything, against a server it starts. Each CI job maps to a target:

| CI job | Local command | What it checks |
| :--- | :--- | :--- |
| Format and lint | `make fmt-check lint doc` | rustfmt; clippy with `-D warnings` and the cognitive-complexity and length lints, with all features and with none; rustdoc with warnings denied |
| Build and unit tests (stable, 1.85.0) | `make build test` | every target, benches included, on stable and the floor |
| Coverage | `make coverage` | 100% of lines and regions in `src/`, or it fails |
| Complexity ceilings | `make complexity` | cyclomatic 10, cognitive 15, Halstead difficulty 30, 60 lines per function, 500 per file |
| Versions, package contents and API breakage | `make versions package-check api-check` | every version restatement; what the crate ships; cargo-semver-checks |
| Dependency policy | `make deny` | RustSec advisories, licences, bans, sources |
| Conformance kit against pain001 serve | `make conformance` and `make bench-smoke` with `PAIN001_BASE_URL` | the kit, the typed tests, every example, every benchmark once |
| Docs lint | `make docs-lint` | markdownlint, codespell, REUSE |
| Manual | `make manual` | MkDocs strict build, API reference, link check |
| Fuzz | see [Fuzzing](#fuzzing) | 30 seconds per target from the seeds |
| CodeQL, Scorecard, DCO | none locally | static analysis, supply chain, `Signed-off-by` |

### Coverage

`make coverage` runs cargo-llvm-cov over the unit and integration tests
and fails below 100% of lines or of regions. The unit tests reach 100%
on their own; the live tests skip without a server. The `cfg(fuzzing)`
decoding module (`src/fuzzing.rs`) is compiled for unit tests and
counted. There are **no exclusions**: no `coverage(off)` attributes and
no ignored files.

llvm-cov counts a generic function by its best single instantiation, so
a test that covers an error path of a generic function has to use a type
whose instantiation also covers the success path (see
`a_body_that_cannot_be_encoded_is_a_decode_error_and_is_not_sent`).

### Against a live server

```sh
uv venv /tmp/pain001-server
uv pip install --python /tmp/pain001-server/bin/python \
  --require-hashes -r ci/server-requirements.txt
/tmp/pain001-server/bin/pain001 serve --host 127.0.0.1 --port 8000 &
PAIN001_BASE_URL=http://127.0.0.1:8000 make conformance
```

Start the server with `PAIN001_API_KEY=<key>` and export the same
variable to exercise the API key path, as CI does. `make regression` does
all of this itself: it installs the server into a temporary venv (or uses
`PAIN001_SERVER`), starts it on a free loopback port with a random key,
runs `make check`, `make conformance` and `make bench-smoke`, and stops
the server and removes the venv even when a step fails.

`ci/server-requirements.txt` pins the latest released pain001
(`ci/server-requirements.in`), hash-locked; regenerate it with the
command in the `.in` file's header.

### Fuzzing

```sh
cargo +nightly fuzz run response fuzz/corpus/response fuzz/seeds/response -- -max_total_time=60
cargo +nightly fuzz run error fuzz/corpus/error fuzz/seeds/error -- -max_total_time=60
```

A crash lands in `fuzz/artifacts/`: fix the cause, add a unit test in the
main crate, and add the input to `fuzz/seeds/`.

### Benchmarks

`make bench` runs both criterion benches; reports land in
`target/criterion/`. `benches/client.rs` needs no server;
`benches/roundtrip.rs` needs `PAIN001_BASE_URL` and skips without it.
`scripts/record-bench-responses.sh` re-records the response bodies the
client bench decodes. See [BENCHMARKS.md](https://github.com/sebastienrousseau/pain001-sdk-rust/blob/main/docs/BENCHMARKS.md).

## Test layout

| Where | What | Needs a server |
| :--- | :--- | :--- |
| `src/**` `#[cfg(test)]` modules, `src/client/tests.rs`, `src/retry/tests.rs` | unit tests, against `src/stub.rs`, a one-request loopback stub | no |
| `tests/conformance.rs` | core's conformance kit through `Client::request`, plus offline tests of the runner and the kit's SHA-256 sums | for the kit |
| `tests/typed.rs` | one test per typed method, asserting fields | yes |
| `examples/*.rs` | one program per method; each exits non-zero on a wrong answer | yes |
| `benches/*.rs` | criterion, client-side and round trip | round trips only |
| `fuzz/` | cargo-fuzz targets over response decoding and error mapping | no |

## Release model

Versions advance by `0.0.1`, in lockstep with the pain001 suite; the
maintainer opens a release. A release is prepared, then tagged by the
maintainer, and the [release workflow](https://github.com/sebastienrousseau/pain001-sdk-rust/blob/main/.github/workflows/release.yml)
does the rest:

1. On the release branch: rename `## [Unreleased]` in `CHANGELOG.md` to
   `## [<version>] - <date>`, write `releases/v<version>.md` (two to four
   Highlights), and check `make versions`.
2. Run the workflow by hand first (Actions, Release, Run workflow): a dry
   run builds, packages, writes the SBOM and checksums, and prints what a
   release would carry.
3. After the release pull request is merged, the maintainer creates and
   pushes a signed annotated tag: `git tag -s v<version> -m
   "pain001-sdk-rust v<version>"`.
4. The tag runs the workflow: the GitHub release titled
   `pain001-sdk-rust <version>` with the `.crate`, the CycloneDX SBOM and
   `SHA256SUMS`, provenance attestations, notes composed by
   `scripts/release_notes.py`, then `cargo publish` to crates.io.
5. Check the live release (`gh release view v<version> --json name,body`)
   and the crate on crates.io and docs.rs.

The workflow is safe to re-run: assets are replaced, the notes rewritten,
and a version already on crates.io is reported, not failed.

### Settings the maintainer creates

None of these exist yet, and none are created by automation:

| Setting | Where | Why |
| :--- | :--- | :--- |
| `release` environment | Settings, Environments | the publish job runs in it; add a required reviewer |
| First publish of 0.0.88 | crates.io | crates.io registers a trusted publisher only for a crate that exists: publish 0.0.88 once by hand from the signed tag (`cargo publish --locked`), or set `CRATES_IO_BOOTSTRAP_TOKEN` (a scoped crates.io token) as a `release` environment secret for that one run, then delete it and revoke the token |
| Trusted publisher | crates.io, `pain001-sdk`, Settings, Trusted Publishing | repository `sebastienrousseau/pain001-sdk-rust`, workflow `release.yml`, environment `release` |
| GitHub Pages | Settings, Pages, Source: GitHub Actions | the manual's deploy job |
| `PAGES_ENABLED` = `true` | Settings, Variables, Actions | turns the manual's deploy job on once Pages is enabled |
| OpenSSF Best Practices badge | bestpractices.dev | clears Scorecard's `CIIBestPracticesID` alert |

## Writing code here

Read [AGENTS.md](https://github.com/sebastienrousseau/pain001-sdk-rust/blob/main/AGENTS.md)
and [CONTRIBUTING.md](https://github.com/sebastienrousseau/pain001-sdk-rust/blob/main/CONTRIBUTING.md).
In short: no `unsafe`, no logging of request or response bodies, no
retries of a `POST`, no new runtime dependency without a reason, tests in
the same commit, and every commit signed off (`git commit -s`).
