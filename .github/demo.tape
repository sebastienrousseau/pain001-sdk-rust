# SPDX-License-Identifier: Apache-2.0 OR MIT
#
# Rendered by `make demo` (vhs, https://github.com/charmbracelet/vhs) from
# the repository root. Off screen it starts the pain001 server named by
# PAIN001_SERVER on loopback, in a throwaway directory. On screen it runs
# examples/basic.rs against it: health, generate a pain.001.001.09 file
# from the conformance kit's synthetic CSV, check it, then show the file's
# first lines. Nothing leaves 127.0.0.1. One hidden run first warms the
# server, whose first generate is slow, so the visible run shows its
# usual pace.
Output .github/demo.gif

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 640
Set Padding 20
Set Theme "Catppuccin Mocha"
Set WaitTimeout 120s
Set WaitPattern /\$\s*$/

Hide
Type `export TMPDIR="$(mktemp -d /tmp/pain001-sdk-demo.XXXX)" PAIN001_BASE_URL=http://127.0.0.1:18087`
Enter
Type `(cd "$TMPDIR" && exec "$PAIN001_SERVER" serve --host 127.0.0.1 --port 18087 >server.log 2>&1) & DEMO_SERVER=$!`
Enter
Type `for _ in $(seq 1 60); do curl -fsS "$PAIN001_BASE_URL/api/v1/health" >/dev/null 2>&1 && break; sleep 0.5; done`
Enter
Type `cargo run -q --example basic >/dev/null; export PS1='$ ' && clear`
Enter
Wait
Show

Type "cargo run -q --example basic"
Sleep 500ms
Enter
Wait+Screen /rungs=/
Sleep 6s

Type `head -n 14 "$TMPDIR/pain001-example.xml"`
Sleep 500ms
Enter
Wait
Sleep 12s

Hide
Type `kill "$DEMO_SERVER"; wait "$DEMO_SERVER" 2>/dev/null; rm -rf "$TMPDIR"`
Enter
Sleep 1s
