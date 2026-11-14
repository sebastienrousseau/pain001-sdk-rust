// SPDX-License-Identifier: Apache-2.0 OR MIT
//! The response decoding path without the network, for the cargo-fuzz
//! targets in `fuzz/`. Compiled only under `cfg(fuzzing)` (which
//! cargo-fuzz sets) and for unit tests, so it is not part of the public
//! API of a normal build.

use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::client;
use crate::error::{api_error, generate_error, generated};
use crate::{Error, GenerateResponse};

/// What the client makes of a response body's text: empty is `null`,
/// non-JSON is a JSON string. [`crate::Client::request`] returns this.
pub fn parse_body(text: &str) -> Value {
    client::parse_body(text)
}

/// What a typed call other than `generate` returns for `status` and the
/// body `text`, as if the server had sent them: a 2xx body decodes as
/// `T`, any other status becomes [`Error::Api`].
pub fn decode<T: DeserializeOwned>(status: u16, text: &str) -> Result<T, Error> {
    client::decode(status, client::parse_body(text), api_error)
}

/// What [`crate::Client::generate`] returns for `status` and the body
/// `text`: as [`decode`], except that a `422` refusal body becomes
/// [`Error::Refused`].
pub fn generate(status: u16, text: &str) -> Result<GenerateResponse, Error> {
    client::decode(status, client::parse_body(text), generate_error).and_then(generated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HealthResponse;
    use serde_json::json;

    #[test]
    fn decode_follows_the_client_mapping() {
        let ok: HealthResponse =
            decode(200, r#"{"status":"healthy","version":"0","message":""}"#).expect("decodes");
        assert_eq!(ok.status, "healthy");
        let refused = generate(422, r#"{"success":false,"message":"m"}"#).expect_err("refused");
        assert!(refused.refusal().is_some());
        let plain = decode::<GenerateResponse>(422, r#"{"success":false,"message":"m"}"#)
            .expect_err("api error");
        assert!(matches!(plain, Error::Api { status: 422, .. }));
        assert!(matches!(
            decode::<HealthResponse>(204, ""),
            Err(Error::Decode(_))
        ));
        assert_eq!(parse_body("not json"), json!("not json"));
    }
}
