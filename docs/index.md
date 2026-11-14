<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# pain001-sdk-rust

A typed, blocking Rust client for the pain001 REST API, which generates
and validates ISO 20022 pain.001 and pain.008 payment files. The engine
is the Python server, `pain001 serve`; this crate is a thin client for
it, proven against the suite's conformance kit in CI.

```toml
[dependencies]
pain001-sdk = "0.0.88"
```

```rust
use pain001_sdk::{Client, Error, MessageType};

fn main() -> Result<(), Error> {
    let client = Client::new("http://127.0.0.1:8000");
    let csv = std::fs::read_to_string("payments.csv").expect("read the CSV");
    let generated = client.generate(&csv, "payments.csv", MessageType::Pain00100109, None)?;
    let report = client.check(generated.xml.as_deref().unwrap_or_default(), None, None)?;
    println!("{} passed={}", report.message_type, report.passed);
    Ok(())
}
```

## Where to go

| You want to | Read |
| :--- | :--- |
| Install it and make a first call | [the README](https://github.com/sebastienrousseau/pain001-sdk-rust#readme) |
| Look up a type or method | the API reference on [docs.rs](https://docs.rs/pain001-sdk), or [this site's copy](https://sebastienrousseau.github.io/pain001-sdk-rust/api/pain001_sdk/) built from `main` |
| See every method run | [the examples](https://github.com/sebastienrousseau/pain001-sdk-rust/tree/main/examples) |
| Understand how a call works | [Architecture](ARCHITECTURE.md) and the [decision records](adr/README.md) |
| Compare it with the other SDKs | [Ecosystem comparison](COMPARISON.md) |
| Know what it costs | [Benchmarks](BENCHMARKS.md) |
| Package it | [Packaging](packaging.md) and [policies](POLICIES.md) |
| Work on it | [Development](project/development.md) and [Contributing](project/contributing.md) |
