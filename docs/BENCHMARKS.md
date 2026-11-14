<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Benchmarks

Every public method is benchmarked twice, with
[criterion](https://crates.io/crates/criterion) 0.7:

- **Client-side cost** (`benches/client.rs`, no network): encoding the
  request body each method sends, from the conformance kit's fixtures,
  and decoding the response it reads, from the bodies pain001 0.0.87 sent
  for those fixtures (`benches/responses/`, recorded by
  `scripts/record-bench-responses.sh`). This is the only part of a call
  the crate controls.
- **Loopback round trip** (`benches/roundtrip.rs`): the whole call
  against a local `pain001 serve`, the client's work plus the server's.

```sh
make bench                                            # client-side only
PAIN001_BASE_URL=http://127.0.0.1:8000 make bench     # round trips too
```

CI runs each benchmark once on every pull request (`make bench-smoke`),
so they keep compiling and running; CI asserts no numbers.

## Read this first

The numbers below were measured on a laptop that was running other
builds at the same time; the load average is stated for each run. Run 2,
at about three times run 1's load, is 3 to 10 times slower for the same
code. Treat them as an order of magnitude: client-side work is
microseconds, a round trip is milliseconds, and the server's work
dominates every call that sends a payment file. Re-run `make bench` on an
idle machine before comparing changes.

## Environment

| | |
| :--- | :--- |
| Machine | Apple A18 Pro, 6 cores, 8 GiB, macOS 27.2 |
| Rust | rustc 1.99.0 (stable), `cargo bench` (the `bench` profile, optimised) |
| criterion | 0.7.0, default settings (3 s warm-up, 100 samples) |
| Server | pain001 0.0.87 on Python 3.12.14, `pain001 serve --host 127.0.0.1`, started with `PAIN001_API_KEY` |
| Date | 2026-11-14 |
| Run 1 load average (1 min) | 62.9 at the client-side start, 24.9 to 54.8 over the round trips |
| Run 2 load average (1 min) | 181.7 to 187.1 client-side, 187.1 to 153.9 round trips |

Each cell is criterion's median estimate, with its 95% confidence
interval in brackets.

## Client-side cost

| Benchmark | What it times | Run 1 | Run 2 |
| :--- | :--- | ---: | ---: |
| `encode/validate` | the `validate` body for the kit's 5-row CSV | 3.77 µs [3.03, 4.76] | 14.96 µs [12.65, 17.65] |
| `encode/generate` | the `generate` body, with a scheme | 2.48 µs [2.04, 3.06] | 9.03 µs [7.72, 10.41] |
| `encode/check` | the `check` body for the kit's pain.001 XML | 9.95 µs [8.67, 11.44] | 50.25 µs [41.94, 59.21] |
| `encode/project_face` | the face path and body | 9.49 µs [8.50, 10.72] | 107.82 µs [89.97, 128.19] |
| `encode/request` | a `serde_json::Value` body for `request` | 9.41 µs [8.37, 10.79] | 42.56 µs [33.80, 52.33] |
| `decode/health` | `HealthResponse` (74 bytes) | 1.06 µs [0.85, 1.32] | 2.88 µs [2.27, 3.58] |
| `decode/validate` | `ValidationResponse` (99 bytes) | 1.07 µs [0.91, 1.28] | 4.28 µs [3.65, 4.97] |
| `decode/check` | `CheckResponse` (108 bytes) | 0.84 µs [0.81, 0.86] | 6.92 µs [5.60, 8.39] |
| `decode/generate` | `GenerateResponse` with the XML (4,466 bytes) | 12.03 µs [11.58, 12.54] | 58.16 µs [51.18, 66.05] |
| `decode/generate_refused` | a `422` refusal into `Error::Refused` (220 bytes) | 2.02 µs [1.77, 2.42] | 7.40 µs [6.63, 8.20] |
| `decode/faces` | `Vec<FaceInfo>` (455 bytes) | 2.05 µs [1.89, 2.24] | 5.89 µs [4.98, 6.90] |
| `decode/project_face` | `FaceResponse` (4,491 bytes) | 13.17 µs [12.64, 13.76] | 64.28 µs [57.91, 71.12] |
| `decode/request` | `request`'s parse of the same body to JSON | 9.44 µs [9.14, 9.79] | 42.47 µs [37.60, 47.93] |

## Loopback round trip

| Benchmark | Run 1 | Run 2 |
| :--- | ---: | ---: |
| `roundtrip/health` | 0.44 ms [0.39, 0.50] | 1.13 ms [1.00, 1.27] |
| `roundtrip/faces` | 4.13 ms [3.31, 5.02] | 2.50 ms [2.30, 2.73] |
| `roundtrip/request` (`GET /api/v1/health`) | 3.07 ms [2.04, 4.27] | 1.32 ms [1.16, 1.49] |
| `roundtrip/validate` | 14.07 ms [12.14, 16.17] | 8.16 ms [7.59, 8.80] |
| `roundtrip/generate` | 16.39 ms [13.35, 20.57] | 17.35 ms [15.63, 19.50] |
| `roundtrip/check` | 13.27 ms [10.91, 15.93] | 10.34 ms [9.45, 11.33] |
| `roundtrip/project_face` | 24.18 ms [19.07, 29.97] | 15.65 ms [13.74, 17.81] |

The round trips do not move with load the way the client-side numbers
do, and in places run 2 is faster than run 1: the server and the client
share the machine, and which of them the scheduler favoured varied
between runs. That is the noise this page warns about.

## What the numbers say

- The client's own work for any call is tens of microseconds at most,
  against milliseconds for the round trip: under 1% of a call that sends
  a payment file, even at run 1's numbers.
- The biggest client-side costs scale with the document: encoding the
  XML into a JSON string and decoding the generated or projected
  document back, both a few kilobytes here.
- Every method first parses the body into a `serde_json::Value`, which
  is all `request` does (`decode/request`); the typed methods then
  convert that value into their structs, which adds little
  (`decode/project_face` against `decode/request`, same body).
