# SPDX-License-Identifier: Apache-2.0 OR MIT
# The development container: stable Rust with every tool `make check`
# needs, at the versions CI pins. The base is pinned by digest; Dependabot
# keeps it current (docker ecosystem, .devcontainer).
FROM mcr.microsoft.com/devcontainers/rust:1-bookworm@sha256:99196945fec17043ebd2d802b01c9700a68e51722a009612fbc3ab31c1cf325d

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
