<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# 3. One error type, four kinds

- Status: accepted
- Date: 2026-11-14 (recording decisions made in 0.0.86 and 0.0.87)

## Context

A call can fail because the server refused it, because the request never
completed, or because the reply was not what the API documents. Callers
need to tell these apart without parsing messages, and a refused
generation carries findings worth keeping typed. pain001 0.0.87 answers a
refused generation with HTTP `422` and a body in the success shape.

## Decision

Every method returns `Result<_, pain001_sdk::Error>`, a
`#[non_exhaustive]` enum:

| Variant | When | Carries |
| :--- | :--- | :--- |
| `Api { status, detail }` | any non-2xx answer other than a generate refusal | the status and the body's `detail` (string, or the FastAPI array for a `422`, typed by `Error::request_errors`) |
| `Refused { status, response }` | a generate `422` whose body has `success: false` and no `xml` | the decoded `GenerateResponse`, with typed `validation_errors` and `scheme_violations` |
| `Transport` | no complete answer: connection, timeout, TLS, invalid URL, a body cut short | the source error |
| `Decode` | a 2xx body of the wrong shape, a generate success without the XML, or a request body that cannot be encoded | the serde error |

`Error::status()` covers both HTTP kinds, so code that only wants the
status needs no match. An `Ok` from `generate` always carries the XML.

## Consequences

- A server sending a shape the crate does not know surfaces as `Decode`
  rather than silently missing data; the fuzz targets check that only
  documented variants come out of the decoding path.
- Adding a variant later is not breaking, since the enum is
  `#[non_exhaustive]`.
