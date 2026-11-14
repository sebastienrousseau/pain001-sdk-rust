#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0 OR MIT
#
# Builds the user manual into site/: MkDocs Material from docs/ with
# --strict (a broken nav entry, page link or anchor fails), the rustdoc
# API reference into site/api/, then checks every local link of the
# result with lychee --offline and the author footer on every manual
# page. The Manual workflow runs it; locally, `make manual`.
#
# Needs mkdocs from docs/requirements.txt on PATH (or MKDOCS), cargo,
# and lychee.
set -euo pipefail

cd "$(dirname "$0")/.."

mkdocs="${MKDOCS:-mkdocs}"
rm -rf site
"$mkdocs" build --strict

RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps
target="$(cargo metadata --locked --no-deps --format-version 1 \
  | python3 -I -c 'import json, sys; print(json.load(sys.stdin)["target_directory"])')"
cp -R "$target/doc" site/api
# rustdoc --no-deps writes no index of its own; its help and settings
# pages link to one. Send it to the crate's page.
cat > site/api/index.html <<'HTML'
<!doctype html>
<meta charset="utf-8">
<title>pain001-sdk API reference</title>
<meta http-equiv="refresh" content="0; url=pain001_sdk/index.html">
<a href="pain001_sdk/index.html">pain001_sdk</a>
HTML

# MkDocs Material writes root-relative links under the site's path
# (/pain001-sdk-rust/...); give lychee a root where that path is site/.
root="$(mktemp -d)"
trap 'rm -rf "$root"' EXIT
ln -s "$PWD/site" "$root/pain001-sdk-rust"
lychee --offline --no-progress --root-dir "$root" 'site/**/*.html'

footer='&copy; 2026 <a href="https://sebastienrousseau.com/" rel="author">Sebastien Rousseau</a>'
missing=0
while IFS= read -r page; do
  if ! grep -qF "$footer" "$page"; then
    echo "build-manual: no author footer in $page" >&2
    missing=$((missing + 1))
  fi
done < <(find site -name index.html -not -path 'site/api/*')
if [ "$missing" -ne 0 ]; then
  exit 1
fi
echo "build-manual: site/ built; links and footers checked"
