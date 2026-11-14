<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Contributing

Thanks for looking. This crate is a thin typed client for the pain001
REST API: the engine is the Python server, and this repository only turns
its JSON into Rust types and back.

## Before you open a pull request

Everything CI checks, you can run locally:

```sh
make check
```

That is `cargo fmt --check`, clippy with `-D warnings` and the
complexity lints, the build, the tests at 100% line and region coverage,
`cargo doc` with warnings denied, `cargo deny check`, the complexity
ceilings, the version check and the package contents check.
[DEVELOPMENT.md](https://github.com/sebastienrousseau/pain001-sdk-rust/blob/main/DEVELOPMENT.md)
lists the tools and maps every CI job to its local command;
`make regression` runs everything against a server it starts.

The conformance and typed tests need a server:

```sh
pip install 'pain001[api]==0.0.87'
pain001 serve --host 127.0.0.1 --port 8000 &
PAIN001_BASE_URL=http://127.0.0.1:8000 make conformance
```

Without `PAIN001_BASE_URL` they print a skip message and pass.

The fuzz targets in `fuzz/` need a nightly toolchain and cargo-fuzz
(`cargo install cargo-fuzz --locked`):

```sh
cargo +nightly fuzz run response fuzz/corpus/response fuzz/seeds/response -- -max_total_time=60
cargo +nightly fuzz run error fuzz/corpus/error fuzz/seeds/error -- -max_total_time=60
```

A crash input lands in `fuzz/artifacts/`; fix the cause, add a unit
test for it in the main crate, and add the input to `fuzz/seeds/`.

## The README demo

`.github/demo.gif` is rendered from `.github/demo.tape` with
[vhs](https://github.com/charmbracelet/vhs), never recorded by hand:

```sh
uv venv /tmp/pain001-server
uv pip install --python /tmp/pain001-server/bin/python \
  --require-hashes -r ci/server-requirements.txt
PAIN001_SERVER=/tmp/pain001-server/bin/pain001 make demo
```

`make demo` takes the server binary from `PAIN001_SERVER`, or from
`pain001` on `PATH`, and stops with a message when neither is there. The
tape starts that server on 127.0.0.1 in a throwaway directory, runs
`examples/basic.rs` against it, then stops it; nothing leaves loopback.
Re-render the GIF in the same change as anything that alters what it
shows.

## Rules

- **`conformance/` is vendored from pain001 core.** Never edit it here.
  Re-vendor the whole directory from core's release, then regenerate
  `conformance/SHA256SUMS`; a test fails when the two disagree.
- **Complexity ceilings per function**: cyclomatic 10, cognitive 15,
  60 lines, and 500 lines per file. Clippy enforces cognitive complexity
  and length through `clippy.toml`.
- **No new warnings and no new suppressions.**
- **Tests with the change**, in the same commit.
- **Commits** are conventional (`feat:`, `fix:`, `docs:`), signed, and
  carry a `Signed-off-by:` trailer (`git commit -s`); see `DCO.txt`.

## Versioning

Versions advance by 0.0.1 in lockstep with the pain001 suite. The version
in `Cargo.toml` and `CITATION.cff` moves only when the maintainer opens a
release, together with a `CHANGELOG.md` heading.

## Licence

Apache-2.0 OR MIT, at your option. By contributing you agree your work is
released under the same dual grant.
