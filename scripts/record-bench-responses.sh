#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0 OR MIT
#
# Records the response bodies the client-side benches decode, from a live
# `pain001 serve` and the conformance kit's synthetic fixtures, into
# benches/responses/. Re-run it when the kit or the server version moves,
# and commit the result with the server version in the commit body.
#
#   PAIN001_BASE_URL=http://127.0.0.1:8000 scripts/record-bench-responses.sh
#
# Set PAIN001_API_KEY when the server was started with one.
set -euo pipefail

cd "$(dirname "$0")/.."

base="${PAIN001_BASE_URL:?set PAIN001_BASE_URL to a running pain001 serve}"
out="benches/responses"
kit="conformance/fixtures"
auth=()
if [ -n "${PAIN001_API_KEY:-}" ]; then
  auth=(-H "Authorization: Bearer $PAIN001_API_KEY")
fi

# body FIELD=FILE|FIELD=:VALUE ...: a JSON object, file contents read in.
body() {
  python3 -I -c '
import json, sys
out = {}
for arg in sys.argv[1:]:
    key, _, value = arg.partition("=")
    if value.startswith(":"):
        out[key] = value[1:]
    else:
        with open(value, encoding="utf-8") as f:
            out[key] = f.read()
print(json.dumps(out))' "$@"
}

# fetch NAME METHOD PATH [STATUS] [BODY]: one response, status checked.
fetch() {
  local name="$1" method="$2" path="$3" want="${4:-200}" data="${5:-}"
  local args=(-sS -o "$out/$name.json" -w '%{http_code}' -X "$method" "${auth[@]}")
  if [ -n "$data" ]; then
    args+=(-H 'Content-Type: application/json' --data-binary "$data")
  fi
  local status
  status="$(curl "${args[@]}" "$base$path")"
  if [ "$status" != "$want" ]; then
    echo "record: $method $path answered $status, want $want" >&2
    exit 1
  fi
  echo "record: $name.json ($status, $(wc -c <"$out/$name.json" | tr -d ' ') bytes)"
}

mkdir -p "$out"
csv="$kit/credit-transfer.pain.001.001.09.csv"
xml="$kit/credit-transfer.pain.001.001.09.xml"
mt=":pain.001.001.09"

fetch health GET /api/v1/health
fetch validate POST /api/v1/ui/validate 200 \
  "$(body content="$csv" filename=:payments.csv message_type="$mt")"
fetch generate POST /api/v1/ui/generate 200 \
  "$(body content="$csv" filename=:payments.csv message_type="$mt")"
fetch refusal POST /api/v1/ui/generate 422 \
  "$(body content="$kit/bad-currency.pain.001.001.09.csv" filename=:payments.csv message_type="$mt")"
fetch check POST /api/v1/check 200 "$(body xml="$xml")"
fetch faces GET /api/v1/faces
fetch face POST /api/v1/faces/berlin_group 200 "$(body xml="$xml")"
