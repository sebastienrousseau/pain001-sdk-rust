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
complexity lints, the build, the tests, `cargo doc` with warnings denied
and `cargo deny check` (install it with `cargo install cargo-deny --locked`).

The conformance and typed tests need a server:

```sh
pip install 'pain001[api]==0.0.87'
pain001 serve --host 127.0.0.1 --port 8000 &
PAIN001_BASE_URL=http://127.0.0.1:8000 make conformance
```

Without `PAIN001_BASE_URL` they print a skip message and pass.

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
