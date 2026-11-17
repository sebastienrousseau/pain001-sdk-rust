#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0 OR MIT
#
# API breakage check: cargo-semver-checks against the newest version of
# pain001-sdk on crates.io, held to a patch release's rules, and failing
# if it ran no checks. Until the first release is published there is
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
# Between 0.0.x versions every bump counts as major, so cargo-semver-checks
# would skip every check; --release-type patch makes each 0.0.1 step hold
# the API to a patch release's rules instead.
out="$(cargo semver-checks check-release --package "$crate" \
  --default-features --release-type patch 2>&1)" || {
  printf '%s\n' "$out"
  exit 1
}
printf '%s\n' "$out"

# A run that checks nothing proves nothing: fail rather than pass.
if grep -qE 'Checked .* 0 checks' <<<"$out"; then
  echo "api-check: cargo-semver-checks ran no checks" >&2
  exit 1
fi
