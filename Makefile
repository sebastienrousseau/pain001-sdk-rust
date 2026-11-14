# SPDX-License-Identifier: Apache-2.0 OR MIT
# Local gates. `make check` runs what CI runs; the conformance and typed
# tests reach a live `pain001 serve` only when PAIN001_BASE_URL is set,
# and report themselves skipped otherwise.

CARGO ?= cargo
CLIPPY_FLAGS := -D warnings -W clippy::cognitive_complexity -W clippy::too_many_lines

.PHONY: check fmt fmt-check lint build test doc deny conformance demo clean

check: fmt-check lint build test doc deny

fmt:
	$(CARGO) fmt

fmt-check:
	$(CARGO) fmt --check

lint:
	$(CARGO) clippy --locked --all-targets -- $(CLIPPY_FLAGS)
	$(CARGO) clippy --locked --all-targets --no-default-features -- $(CLIPPY_FLAGS)

build:
	$(CARGO) build --locked --all-targets

test:
	$(CARGO) test --locked

doc:
	RUSTDOCFLAGS="-D warnings" $(CARGO) doc --locked --no-deps

# Needs cargo-deny (`cargo install cargo-deny --locked`).
deny:
	$(CARGO) deny --locked check

# The kit against a live server: PAIN001_BASE_URL=http://127.0.0.1:8000 make conformance
conformance:
	@test -n "$(PAIN001_BASE_URL)" || { echo "set PAIN001_BASE_URL"; exit 1; }
	$(CARGO) test --locked --test conformance --test typed -- --nocapture

# Renders .github/demo.gif from .github/demo.tape (needs vhs). The tape
# starts the pain001 server on 127.0.0.1: set PAIN001_SERVER to its
# binary, or put `pain001` on PATH (install ci/server-requirements.txt).
# If vhs fails part way, the server it started is stopped here.
PAIN001_SERVER ?= $(shell command -v pain001 2>/dev/null)

demo:
	@test -n "$(PAIN001_SERVER)" && test -x "$(PAIN001_SERVER)" || { \
	  echo "make demo needs the pain001 server: set PAIN001_SERVER=/path/to/pain001"; \
	  echo "or put pain001 on PATH (see CONTRIBUTING.md)"; exit 1; }
	@command -v vhs >/dev/null || { echo "make demo needs vhs (https://github.com/charmbracelet/vhs)"; exit 1; }
	$(CARGO) build --locked --example basic
	PAIN001_SERVER="$(PAIN001_SERVER)" vhs .github/demo.tape; status=$$?; \
	  pkill -f "$(PAIN001_SERVER) serve --host 127.0.0.1 --port 18087" || true; \
	  exit $$status

clean:
	$(CARGO) clean
