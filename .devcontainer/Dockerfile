# SPDX-License-Identifier: Apache-2.0 OR MIT
# The development container: stable Rust with every tool `make check`
# needs, at the versions CI pins. The base is pinned by digest; Dependabot
# keeps it current (docker ecosystem, .devcontainer).
FROM mcr.microsoft.com/devcontainers/rust:1-bookworm@sha256:86d9228ec0744191b45ec91e2bb9a0dd7d7bfd4c4f01ef47abb78032ddddf1c7

RUN apt-get update \
  && apt-get install -y --no-install-recommends python3 python3-venv \
  && rm -rf /var/lib/apt/lists/*

USER vscode
RUN rustup component add clippy rustfmt llvm-tools-preview \
  && cargo install --locked \
    cargo-deny@0.20.2 \
    cargo-llvm-cov@0.9.1 \
    rust-code-analysis-cli@0.0.25 \
  && rm -rf "$CARGO_HOME/registry" "$CARGO_HOME/git"
