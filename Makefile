# SPDX-License-Identifier: Apache-2.0 OR MIT
# Local gates. `make check` runs what CI runs; the conformance and typed
# tests reach a live `pain001 serve` only when PAIN001_BASE_URL is set,
# and report themselves skipped otherwise.

CARGO ?= cargo
CLIPPY_FLAGS := -D warnings -W clippy::cognitive_complexity -W clippy::too_many_lines

.PHONY: check fmt fmt-check lint build test doc deny conformance clean

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

clean:
	$(CARGO) clean
