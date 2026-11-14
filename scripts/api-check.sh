#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0 OR MIT
#
# API breakage check: cargo-semver-checks against the newest version of
# pain001-sdk on crates.io. Until the first release is published there is
# nothing to compare with, so the check reports that and passes; from the
# first release on it runs on every pull request.
#
#   scripts/api-check.sh
set -euo pipefail

cd "$(dirname "$0")/.."

crate="pain001-sdk"
status="$(curl -sS -o /dev/null -w '%{http_code}' \
  -A "pain001-sdk-rust api-check (https://github.com/sebastienrousseau/pain001-sdk-rust)" \
  "https://crates.io/api/v1/crates/$crate")"
case "$status" in
  404)
    echo "api-check: $crate is not on crates.io yet; nothing to compare with"
    exit 0
    ;;
  200) ;;
  *)
    echo "api-check: crates.io answered $status for $crate" >&2
    exit 1
    ;;
esac

# The default features: what a user gets. The `internals` feature is not
# public API, so its hidden module is left out of the comparison.
cargo semver-checks check-release --package "$crate" --default-features
