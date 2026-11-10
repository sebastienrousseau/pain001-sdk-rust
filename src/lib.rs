// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Typed blocking client for the pain001 REST API.
//!
//! [pain001](https://pain001.com) generates and validates ISO 20022
//! `pain.001` (credit transfer) and `pain.008` (direct debit) payment
//! files. Its engine is a Python server, `pain001 serve`; this crate is a
//! thin typed client for it. Every request and response is JSON, sent with
//! [`ureq`] on the calling thread, with no async runtime.
//!
//! ```no_run
//! use pain001_sdk::{Client, MessageType};
//! use std::time::Duration;
//!
//! # fn main() -> Result<(), pain001_sdk::Error> {
//! let client = Client::new("http://127.0.0.1:8000").timeout(Duration::from_secs(10));
//! let csv = std::fs::read_to_string("payments.csv").expect("read the CSV");
//! let generated = client.generate(&csv, "payments.csv", MessageType::Pain00100109, None)?;
//! assert!(generated.success);
//! # Ok(())
//! # }
//! ```
//!
//! A non-2xx response becomes [`Error::Api`], carrying the HTTP status and
//! the server's `detail` (a string, or an array for a `422`). The client
//! never retries and never logs request bodies, which hold payment data.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod client;
mod error;
mod types;

#[cfg(test)]
mod stub;

pub use client::{Client, DEFAULT_TIMEOUT};
pub use error::Error;
pub use types::{
    CheckFinding, CheckResponse, FaceInfo, FaceResponse, GenerateResponse, HealthResponse,
    LossReport, MessageType, RequestError, RowError, ValidationResponse,
};
