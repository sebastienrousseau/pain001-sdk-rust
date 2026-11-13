// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Request and response shapes, from `conformance/openapi.json`.
//!
//! Response structs are `#[non_exhaustive]` so a field the server adds
//! later is not a breaking change here.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The ISO 20022 message types the server generates and validates.
///
/// Methods take `impl AsRef<str>`, so a string the enum does not list yet
/// (one a newer server supports) still works.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MessageType {
    /// `pain.001.001.03`
    Pain00100103,
    /// `pain.001.001.04`
    Pain00100104,
    /// `pain.001.001.05`
    Pain00100105,
    /// `pain.001.001.06`
    Pain00100106,
    /// `pain.001.001.07`
    Pain00100107,
    /// `pain.001.001.08`
    Pain00100108,
    /// `pain.001.001.09`
    Pain00100109,
    /// `pain.001.001.10`
    Pain00100110,
    /// `pain.001.001.11`
    Pain00100111,
    /// `pain.001.001.12`
    Pain00100112,
    /// `pain.001.001.13`
    Pain00100113,
    /// `pain.008.001.02`
    Pain00800102,
    /// `pain.008.001.08`
    Pain00800108,
}

impl MessageType {
    /// Every message type, in the order of the OpenAPI enum.
    pub const ALL: [MessageType; 13] = [
        MessageType::Pain00100103,
        MessageType::Pain00100104,
        MessageType::Pain00100105,
        MessageType::Pain00100106,
        MessageType::Pain00100107,
        MessageType::Pain00100108,
        MessageType::Pain00100109,
        MessageType::Pain00100110,
        MessageType::Pain00100111,
        MessageType::Pain00100112,
        MessageType::Pain00100113,
        MessageType::Pain00800102,
        MessageType::Pain00800108,
    ];

    /// The wire string, for example `"pain.001.001.09"`.
    pub fn as_str(self) -> &'static str {
        WIRE_NAMES[self as usize]
    }
}

/// Wire strings, indexed by discriminant (the order of [`MessageType::ALL`]).
const WIRE_NAMES: [&str; 13] = [
    "pain.001.001.03",
    "pain.001.001.04",
    "pain.001.001.05",
    "pain.001.001.06",
    "pain.001.001.07",
    "pain.001.001.08",
    "pain.001.001.09",
    "pain.001.001.10",
    "pain.001.001.11",
    "pain.001.001.12",
    "pain.001.001.13",
    "pain.008.001.02",
    "pain.008.001.08",
];

impl AsRef<str> for MessageType {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// `GET /api/v1/health`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct HealthResponse {
    /// `"healthy"` when the server is up.
    pub status: String,
    /// The server's pain001 version.
    pub version: String,
    /// A human-readable message.
    pub message: String,
}

/// One field error in a payment file's rows.
///
/// This is pain001's row error (`field`, `message`, `value`). The
/// pain001 0.0.85 OpenAPI document names FastAPI's request-error schema
/// for these lists by a name collision; the server sends this shape.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct RowError {
    /// The field name or JSON path.
    pub field: String,
    /// What is wrong.
    pub message: String,
    /// The offending value, when the server reports it.
    #[serde(default)]
    pub value: Option<Value>,
}

/// One item of a `422` response's `detail` array: FastAPI's request
/// validation error. See [`crate::Error::request_errors`].
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct RequestError {
    /// Where in the request: for example `["body", "message_type"]`.
    pub loc: Vec<Value>,
    /// What is wrong.
    pub msg: String,
    /// The error kind, for example `enum` or `missing`.
    #[serde(rename = "type")]
    pub kind: String,
    /// The rejected input, when reported.
    #[serde(default)]
    pub input: Option<Value>,
    /// Extra context, when reported.
    #[serde(default)]
    pub ctx: Option<Value>,
}

/// `POST /api/v1/ui/validate`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct ValidationResponse {
    /// Whether every row passed, and every requested scheme rulebook.
    pub is_valid: bool,
    /// Rows read from the file.
    pub total_rows: u64,
    /// Rows that passed.
    #[serde(default)]
    pub valid_rows: u64,
    /// Rows that failed.
    #[serde(default)]
    pub invalid_rows: u64,
    /// One entry per field error in the rows.
    #[serde(default)]
    pub errors: Vec<RowError>,
    /// Scheme-rulebook violations, when a scheme was requested.
    #[serde(default)]
    pub scheme_violations: Vec<Map<String, Value>>,
}

/// `POST /api/v1/ui/generate`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct GenerateResponse {
    /// Whether the XML was generated.
    pub success: bool,
    /// A human-readable result message.
    pub message: String,
    /// The suggested download name for the XML.
    #[serde(default)]
    pub filename: Option<String>,
    /// The generated XML document, on success.
    #[serde(default)]
    pub xml: Option<String>,
    /// Field errors in the rows when generation was refused.
    #[serde(default)]
    pub validation_errors: Vec<RowError>,
    /// Scheme-rulebook violations when generation was refused.
    #[serde(default)]
    pub scheme_violations: Vec<Map<String, Value>>,
}

/// One problem a rung of the validation ladder found.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct CheckFinding {
    /// `xsd`, `mdr`, `scheme` or `schematron`.
    pub rung: String,
    /// The rule identifier.
    pub rule: String,
    /// `error` or `warning`.
    pub severity: String,
    /// What is wrong.
    pub message: String,
    /// Element path, context, or transaction number.
    #[serde(default)]
    pub location: Option<String>,
}

/// `POST /api/v1/check`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct CheckResponse {
    /// The detected message type, for example `pain.001.001.09`.
    pub message_type: String,
    /// `false` when a rung found an error.
    pub passed: bool,
    /// Number of error findings.
    pub errors: u64,
    /// Number of warning findings.
    pub warnings: u64,
    /// The rungs that ran, in order.
    pub rungs: Vec<String>,
    /// Every finding.
    pub findings: Vec<CheckFinding>,
}

/// One entry of `GET /api/v1/faces`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct FaceInfo {
    /// The face name, used in the path of [`crate::Client::project_face`].
    pub name: String,
    /// The target standard.
    pub description: String,
    /// Binding keys the face accepts; any other is refused.
    pub bindings: Vec<String>,
}

/// What a projection carried and what it did not, by indexed path.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct LossReport {
    /// The face projected into.
    pub face: String,
    /// Source leaves counted.
    pub total_source_elements: u64,
    /// Leaves carried into the payload.
    pub mapped_count: u64,
    /// Leaves not carried.
    pub unmapped_count: u64,
    /// `true` when nothing was dropped.
    pub is_lossless: bool,
    /// Share of source leaves not carried.
    pub loss_ratio: f64,
    /// Paths carried.
    pub mapped_paths: Vec<String>,
    /// Paths not carried.
    pub unmapped_paths: Vec<String>,
}

/// `POST /api/v1/faces/{face}`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct FaceResponse {
    /// The face projected into.
    pub face: String,
    /// The source pain.001 edition.
    pub version: String,
    /// The body in the target's shape, as raw JSON.
    pub payload: Value,
    /// What the projection did not carry.
    pub loss_report: LossReport,
}

/// The body of `ui/validate` and `ui/generate`.
#[derive(Serialize)]
pub(crate) struct FilePayload<'a> {
    pub content: &'a str,
    pub filename: &'a str,
    pub message_type: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheme: Option<&'a str>,
}

/// The body of `check`.
#[derive(Serialize)]
pub(crate) struct CheckRequest<'a> {
    pub xml: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheme: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schematron: Option<&'a str>,
}

/// The body of `faces/{face}`.
#[derive(Serialize)]
pub(crate) struct FaceRequest<'a> {
    pub xml: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bindings: Option<&'a Map<String, Value>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_types_match_the_openapi_enum() {
        let spec: Value = serde_json::from_str(include_str!("../conformance/openapi.json"))
            .expect("openapi.json parses");
        let wanted: Vec<&str> = spec["components"]["schemas"]["MessageType"]["enum"]
            .as_array()
            .expect("MessageType enum")
            .iter()
            .filter_map(Value::as_str)
            .collect();
        let ours: Vec<&str> = MessageType::ALL.iter().map(|m| m.as_str()).collect();
        assert_eq!(ours, wanted);
        assert_eq!(MessageType::Pain00800108.to_string(), "pain.008.001.08");
        assert_eq!(MessageType::Pain00100109.as_ref(), "pain.001.001.09");
    }

    #[test]
    fn optional_request_fields_are_left_out() {
        let body = FilePayload {
            content: "a",
            filename: "p.csv",
            message_type: "pain.001.001.09",
            scheme: None,
        };
        let json = serde_json::to_value(&body).expect("serialises");
        assert_eq!(json.as_object().map(Map::len), Some(3));
        let check = CheckRequest {
            xml: "<x/>",
            scheme: Some("sepa-sct"),
            schematron: None,
        };
        let json = serde_json::to_value(&check).expect("serialises");
        assert_eq!(
            json,
            serde_json::json!({"xml": "<x/>", "scheme": "sepa-sct"})
        );
    }

    #[test]
    fn validation_response_defaults_the_optional_fields() {
        let parsed: ValidationResponse =
            serde_json::from_str(r#"{"is_valid": false, "total_rows": 2, "extra": 1}"#)
                .expect("decodes");
        assert_eq!(parsed.valid_rows, 0);
        assert!(parsed.errors.is_empty() && parsed.scheme_violations.is_empty());
    }

    #[test]
    fn row_errors_decode_the_server_shape() {
        let parsed: GenerateResponse = serde_json::from_str(
            r#"{"success": false, "message": "invalid",
                "validation_errors": [{"field": "currency", "message": "bad", "value": "EUX"},
                                      {"field": "id", "message": "missing", "value": null}]}"#,
        )
        .expect("decodes");
        assert_eq!(parsed.validation_errors[0].field, "currency");
        assert_eq!(parsed.validation_errors[0].value, Some(Value::from("EUX")));
        assert_eq!(parsed.validation_errors[1].value, None);
    }
}
