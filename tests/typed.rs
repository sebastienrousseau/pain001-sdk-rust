// SPDX-License-Identifier: Apache-2.0 OR MIT
//! One test per SDK method against the server at `PAIN001_BASE_URL`,
//! asserting the typed fields, with `PAIN001_API_KEY` as the bearer token
//! when it is set. Each test is skipped, with a message, when
//! `PAIN001_BASE_URL` is unset.

use std::fs;
use std::path::Path;

use pain001_sdk::{Client, Error, MessageType};
use serde_json::{json, Map};

const CREDIT_CSV: &str = "credit-transfer.pain.001.001.09.csv";
const CREDIT_XML: &str = "credit-transfer.pain.001.001.09.xml";

fn live() -> Option<Client> {
    match std::env::var("PAIN001_BASE_URL") {
        // A server started with PAIN001_API_KEY wants the key on every call.
        Ok(url) => Some(match std::env::var("PAIN001_API_KEY") {
            Ok(key) if !key.is_empty() => Client::new(url).api_key(key),
            _ => Client::new(url),
        }),
        Err(_) => {
            eprintln!("skipped: set PAIN001_BASE_URL to run the typed tests");
            None
        }
    }
}

fn fixture(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("conformance/fixtures");
    fs::read_to_string(path.join(name)).expect("read fixture")
}

#[test]
fn health_reports_healthy() {
    let Some(client) = live() else { return };
    let health = client.health().expect("health");
    assert_eq!(health.status, "healthy");
    assert!(!health.version.is_empty());
}

#[test]
fn validate_accepts_the_credit_transfer() {
    let Some(client) = live() else { return };
    let report = client
        .validate(
            &fixture(CREDIT_CSV),
            "payments.csv",
            MessageType::Pain00100109,
            None,
        )
        .expect("validate");
    assert!(report.is_valid);
    assert_eq!(
        (report.total_rows, report.valid_rows, report.invalid_rows),
        (5, 5, 0)
    );
    assert!(report.errors.is_empty());
}

#[test]
fn validate_refuses_a_bad_iban_with_400() {
    let Some(client) = live() else { return };
    let bad = fixture("bad-iban.pain.001.001.09.csv");
    let error = client
        .validate(&bad, "payments.csv", "pain.001.001.09", None)
        .expect_err("bad IBAN");
    match error {
        Error::Api { status, detail } => {
            assert_eq!(status, 400);
            let text = detail.as_str().expect("a string detail");
            assert!(text.contains("creditor_account_IBAN") && text.contains("not a valid IBAN"));
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn an_unknown_message_type_is_a_422_with_an_array_detail() {
    let Some(client) = live() else { return };
    let error = client
        .validate(
            &fixture(CREDIT_CSV),
            "payments.csv",
            "pain.999.001.01",
            None,
        )
        .expect_err("unknown type");
    assert_eq!(error.status(), Some(422));
    let items = error.request_errors().expect("typed 422 detail");
    assert_eq!(items[0].loc, [json!("body"), json!("message_type")]);
}

#[test]
fn generate_returns_the_credit_transfer_xml() {
    let Some(client) = live() else { return };
    let generated = client
        .generate(
            &fixture(CREDIT_CSV),
            "payments.csv",
            MessageType::Pain00100109,
            None,
        )
        .expect("generate");
    assert!(generated.success);
    let xml = generated.xml.expect("xml");
    assert!(xml.contains("<NbOfTxs>5</NbOfTxs>") && xml.contains("<CtrlSum>1100.00</CtrlSum>"));
}

#[test]
fn generate_returns_the_direct_debit_xml() {
    let Some(client) = live() else { return };
    let csv = fixture("direct-debit.pain.008.001.08.csv");
    let generated = client
        .generate(&csv, "collections.csv", MessageType::Pain00800108, None)
        .expect("generate");
    assert!(generated.success);
    assert!(generated.xml.expect("xml").contains("<NbOfTxs>2</NbOfTxs>"));
}

#[test]
fn generate_refuses_bad_rows_with_typed_findings() {
    let Some(client) = live() else { return };
    let error = client
        .generate(
            &fixture("bad-currency.pain.001.001.09.csv"),
            "payments.csv",
            MessageType::Pain00100109,
            None,
        )
        .expect_err("refused");
    assert_eq!(error.status(), Some(422));
    let refusal = error.refusal().expect("typed refusal");
    assert!(!refusal.success && refusal.xml.is_none());
    assert!(refusal.message.contains("Validation failed"));
    assert!(refusal.scheme_violations.is_empty());
    assert_eq!(refusal.validation_errors[0].field, "$.currency");
    assert_eq!(refusal.validation_errors[0].value, Some(json!("EURO")));
}

#[test]
fn generate_refuses_a_scheme_breach_with_typed_findings() {
    let Some(client) = live() else { return };
    let error = client
        .generate(
            &fixture("chaps-no-purpose.pain.001.001.09.csv"),
            "payments.csv",
            MessageType::Pain00100109,
            Some("uk-chaps"),
        )
        .expect_err("refused");
    let Error::Refused { status, response } = error else {
        panic!("unexpected {error:?}");
    };
    assert_eq!(status, 422);
    assert!(response.validation_errors.is_empty());
    let violation = &response.scheme_violations[0];
    assert_eq!(
        (
            &violation["rule"],
            &violation["severity"],
            &violation["field"]
        ),
        (
            &json!("UK-CHAPS-PURP"),
            &json!("error"),
            &json!("purpose_code")
        )
    );
}

#[test]
fn check_passes_the_valid_document() {
    let Some(client) = live() else { return };
    let report = client
        .check(&fixture(CREDIT_XML), None, None)
        .expect("check");
    assert!(report.passed);
    assert_eq!(
        (report.message_type.as_str(), report.errors),
        ("pain.001.001.09", 0)
    );
    assert!(!report.rungs.is_empty());
}

#[test]
fn check_refuses_a_document_that_is_not_a_payment() {
    let Some(client) = live() else { return };
    let error = client.check("<x/>", None, None).expect_err("not a payment");
    assert_eq!(error.status(), Some(422));
}

#[test]
fn faces_lists_the_three_faces() {
    let Some(client) = live() else { return };
    let names: Vec<String> = client
        .faces()
        .expect("faces")
        .into_iter()
        .map(|f| f.name)
        .collect();
    assert_eq!(names, ["berlin_group", "iso_2018", "uk_open_banking"]);
}

#[test]
fn project_face_carries_the_debtor_iban() {
    let Some(client) = live() else { return };
    let projected = client
        .project_face("berlin_group", &fixture(CREDIT_XML), None, None)
        .expect("project");
    assert_eq!(
        (projected.face.as_str(), projected.version.as_str()),
        ("berlin_group", "pain.001.001.09")
    );
    assert_eq!(
        projected.payload["debtorAccount"]["iban"],
        "DE07512108001245126162"
    );
    assert_eq!(projected.loss_report.face, "berlin_group");
}

#[test]
fn project_face_refuses_an_unknown_face_and_binding() {
    let Some(client) = live() else { return };
    let xml = fixture(CREDIT_XML);
    let error = client
        .project_face("nope", &xml, None, None)
        .expect_err("unknown");
    assert_eq!(error.status(), Some(404));
    let mut bindings = Map::new();
    bindings.insert("NotABinding".into(), json!("x"));
    let error = client
        .project_face(
            "berlin_group",
            &xml,
            Some("pain.001.001.09"),
            Some(&bindings),
        )
        .expect_err("binding refused");
    assert_eq!(error.status(), Some(422));
}

#[test]
fn request_returns_status_and_body() {
    let Some(client) = live() else { return };
    let (status, body) = client
        .request("GET", "/api/v1/health", None)
        .expect("request");
    assert_eq!((status, body["status"].as_str()), (200, Some("healthy")));
    let (status, _) = client
        .request("GET", "/api/v1/nope", None)
        .expect("request");
    assert_eq!(status, 404);
}

#[test]
fn validate_reports_row_errors_in_the_server_shape() {
    let Some(client) = live() else { return };
    let csv = fixture(CREDIT_CSV).replacen(",EUR,", ",EURO,", 1);
    let report = client
        .validate(&csv, "payments.csv", MessageType::Pain00100109, None)
        .expect("validate");
    assert!(!report.is_valid);
    assert_eq!(report.invalid_rows, 1);
    assert_eq!(report.errors[0].field, "$.currency");
    assert_eq!(report.errors[0].value, Some(json!("EURO")));
}
