// SPDX-License-Identifier: Apache-2.0 OR MIT
//! The one error type every client call returns.

use serde_json::Value;

use crate::types::{GenerateResponse, RequestError};

/// What went wrong with a call.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The server answered with a non-2xx status.
    ///
    /// `detail` is the response's `detail` field when the body is a JSON
    /// object that has one (a string, or an array of field errors for a
    /// `422`), the whole JSON body otherwise, or the raw text as a JSON
    /// string when the body is not JSON.
    #[error("pain001 API returned HTTP {status}: {detail}")]
    Api {
        /// The HTTP status code.
        status: u16,
        /// The server's explanation, parsed from the body.
        detail: Value,
    },
    /// The server refused to generate a document: a `422` from a generate
    /// call whose body is a [`GenerateResponse`] with `success: false`.
    ///
    /// `response` carries the findings, typed: `validation_errors` for rows
    /// that failed field checks and `scheme_violations` for scheme-rulebook
    /// breaches. [`Error::status`] still returns the HTTP status.
    #[error("pain001 API refused the file (HTTP {status}): {}", response.message)]
    Refused {
        /// The HTTP status code, `422`.
        status: u16,
        /// The decoded refusal, with `success: false` and no `xml`.
        response: Box<GenerateResponse>,
    },
    /// The request did not complete: connection refused, timeout, TLS,
    /// an invalid URL, or a body that could not be read.
    #[error("pain001 API request failed: {0}")]
    Transport(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    /// A 2xx response body did not match the expected shape.
    #[error("pain001 API response could not be decoded: {0}")]
    Decode(#[source] serde_json::Error),
}

impl Error {
    /// The HTTP status for [`Error::Api`], `None` for the other kinds.
    pub fn status(&self) -> Option<u16> {
        match self {
            Error::Api { status, .. } | Error::Refused { status, .. } => Some(*status),
            _ => None,
        }
    }

    /// The refused generation for [`Error::Refused`], `None` for the other
    /// kinds: its `validation_errors` and `scheme_violations` say why.
    pub fn refusal(&self) -> Option<&GenerateResponse> {
        match self {
            Error::Refused { response, .. } => Some(response),
            _ => None,
        }
    }

    /// The items of a `422` response's `detail` array, typed: `Some` only
    /// for [`Error::Api`] whose `detail` is an array of FastAPI request
    /// errors (for example an unknown message type).
    pub fn request_errors(&self) -> Option<Vec<RequestError>> {
        match self {
            Error::Api { detail, .. } if detail.is_array() => {
                serde_json::from_value(detail.clone()).ok()
            }
            _ => None,
        }
    }

    pub(crate) fn transport<E>(error: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Error::Transport(Box::new(error))
    }
}

/// Builds the error for a non-2xx reply to a generate call: a `422` whose
/// body decodes as a [`GenerateResponse`] with `success: false` becomes
/// [`Error::Refused`], anything else goes through [`api_error`].
pub(crate) fn generate_error(status: u16, body: Value) -> Error {
    if status == 422 && body.get("success") == Some(&Value::Bool(false)) {
        if let Ok(response) = serde_json::from_value::<GenerateResponse>(body.clone()) {
            return Error::Refused {
                status,
                response: Box::new(response),
            };
        }
    }
    api_error(status, body)
}

/// Builds [`Error::Api`] from a non-2xx status and its parsed body.
pub(crate) fn api_error(status: u16, body: Value) -> Error {
    let detail = match body {
        Value::Object(mut map) if map.contains_key("detail") => {
            map.remove("detail").unwrap_or(Value::Null)
        }
        other => other,
    };
    Error::Api { status, detail }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn api_error_unwraps_the_detail_field() {
        let error = api_error(400, json!({"detail": "bad IBAN"}));
        assert_eq!(error.status(), Some(400));
        assert!(matches!(error, Error::Api { detail, .. } if detail == json!("bad IBAN")));
    }

    #[test]
    fn api_error_keeps_a_body_without_detail() {
        let error = api_error(500, json!({"error": "boom"}));
        assert!(matches!(error, Error::Api { detail, .. } if detail == json!({"error": "boom"})));
        let error = api_error(502, json!("Bad Gateway"));
        assert!(matches!(error, Error::Api { detail, .. } if detail == json!("Bad Gateway")));
    }

    #[test]
    fn display_names_the_status_and_other_kinds_have_none() {
        let error = api_error(404, json!({"detail": "Unknown face"}));
        assert_eq!(
            error.to_string(),
            "pain001 API returned HTTP 404: \"Unknown face\""
        );
        let decode = serde_json::from_str::<u8>("x").unwrap_err();
        assert_eq!(Error::Decode(decode).status(), None);
        let io = std::io::Error::other("refused");
        let transport = Error::transport(io);
        assert_eq!(transport.status(), None);
        assert!(std::error::Error::source(&transport).is_some());
        assert!(transport.request_errors().is_none());
    }

    #[test]
    fn generate_error_types_only_a_422_refusal_body() {
        let refusal = json!({"success": false, "message": "Validation failed", "xml": null,
                             "validation_errors": [], "scheme_violations": []});
        let error = generate_error(422, refusal.clone());
        assert_eq!(error.status(), Some(422));
        assert!(error.request_errors().is_none());
        assert_eq!(error.refusal().map(|r| r.success), Some(false));
        // Another status, a success body, or a shape that does not decode
        // keeps the plain API error with the whole body.
        assert!(generate_error(400, refusal).refusal().is_none());
        let success = generate_error(422, json!({"success": true, "message": "ok"}));
        assert!(success.refusal().is_none());
        let broken = generate_error(422, json!({"success": false}));
        assert!(
            matches!(broken, Error::Api { status: 422, detail } if detail == json!({"success": false}))
        );
        assert!(api_error(422, json!({})).refusal().is_none());
    }

    #[test]
    fn request_errors_type_a_422_detail_array() {
        let error = api_error(
            422,
            json!({"detail": [{"type": "enum", "loc": ["body", "message_type"],
                               "msg": "Input should be ...", "input": "pain.999"}]}),
        );
        let items = error.request_errors().expect("typed");
        assert_eq!(items[0].kind, "enum");
        assert_eq!(items[0].loc, [json!("body"), json!("message_type")]);
        assert_eq!(items[0].ctx, None);
        assert!(api_error(400, json!({"detail": "x"}))
            .request_errors()
            .is_none());
        assert!(api_error(422, json!({"detail": [1]}))
            .request_errors()
            .is_none());
    }
}
