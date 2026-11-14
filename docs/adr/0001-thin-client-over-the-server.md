<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# 1. A thin client over `pain001 serve`

- Status: accepted
- Date: 2026-11-14 (recording a decision made with the first commit)

## Context

pain001 generates and validates ISO 20022 pain.001 and pain.008 files.
Its rules (schemas, the Message Definition Report, scheme rulebooks,
Schematron, identifier checks) are large, change with each release, and
are maintained in one Python codebase. The suite offers SDKs in five
languages.

Porting the engine to Rust would give an in-process library, and five
engines that drift apart.

## Decision

The crate is a client of the REST API that `pain001 serve` exposes. It
holds no payment logic: it encodes requests, decodes responses into
types and maps errors. Every SDK in the suite passes one conformance kit,
vendored from core, against a live server in CI.

## Consequences

- A Rust program needs a reachable pain001 server; the README says so in
  "When not to use".
- A new rule or message type in the server reaches Rust users without a
  crate release; response types are `#[non_exhaustive]` and methods take
  any message type string so the client does not get in the way.
- The kit, not the crate, defines correct behaviour; a kit change is a
  coordinated release across the suite.
