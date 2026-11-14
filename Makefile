# SPDX-License-Identifier: Apache-2.0 OR MIT
# Local gates. `make check` runs what CI runs; the conformance and typed
# tests reach a live `pain001 serve` only when PAIN001_BASE_URL is set,
# and report themselves skipped otherwise.

CARGO ?= cargo
CLIPPY_FLAGS := -D warnings -W clippy::cognitive_complexity -W clippy::too_many_lines

.PHONY: check fmt fmt-check lint build test coverage doc deny conformance examples bench bench-smoke regression demo clean

check: fmt-check lint build test coverage doc deny

fmt:
	$(CARGO) fmt

fmt-check:
	$(CARGO) fmt --check

lint:
	$(CARGO) clippy --locked --all-targets --all-features -- $(CLIPPY_FLAGS)
	$(CARGO) clippy --locked --all-targets --no-default-features -- $(CLIPPY_FLAGS)

build:
	$(CARGO) build --locked --all-targets --all-features

test:
	$(CARGO) test --locked

# 100% of lines and regions in src/, or the target fails. Needs
# cargo-llvm-cov (`cargo install cargo-llvm-cov --locked`) and the
# llvm-tools-preview component. The live tests add nothing without
# PAIN001_BASE_URL: the unit tests alone reach 100%.
coverage:
	$(CARGO) llvm-cov --locked --lib --tests --fail-under-lines 100 --fail-under-regions 100

doc:
	RUSTDOCFLAGS="-D warnings" $(CARGO) doc --locked --no-deps

# Needs cargo-deny (`cargo install cargo-deny --locked`).
deny:
	$(CARGO) deny --locked check

# The kit, the typed tests and every example against a live server:
#   PAIN001_BASE_URL=http://127.0.0.1:8000 make conformance
# Set PAIN001_API_KEY too when the server was started with one.
conformance:
	@test -n "$(PAIN001_BASE_URL)" || { echo "set PAIN001_BASE_URL"; exit 1; }
	$(CARGO) test --locked --test conformance --test typed -- --nocapture
	$(MAKE) examples

# Every program in examples/, each exiting non-zero when the server does
# not answer as it expects.
EXAMPLES := $(sort $(basename $(notdir $(wildcard examples/*.rs))))

examples:
	@test -n "$(PAIN001_BASE_URL)" || { echo "set PAIN001_BASE_URL"; exit 1; }
	$(CARGO) build --locked --examples
	@for example in $(EXAMPLES); do \
	  echo "== example $$example"; \
	  $(CARGO) run --quiet --locked --example $$example || exit 1; \
	done

# Every benchmark (criterion): benches/client.rs, client-side cost with no
# network, and benches/roundtrip.rs, a loopback round trip per method
# against PAIN001_BASE_URL (skipped without it). Reports land in
# target/criterion/. Pass criterion options with BENCH_ARGS.
bench:
	$(CARGO) bench --locked --all-features -- $(BENCH_ARGS)

# Each benchmark once, unmeasured: keeps them compiling and running.
bench-smoke:
	$(CARGO) bench --locked --all-features -- --test

# Everything, against a server it starts and stops: scripts/regression.sh.
# Installs the pinned server into a temporary venv unless PAIN001_SERVER
# names one; REGRESSION_PORT picks the loopback port.
regression:
	scripts/regression.sh

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
