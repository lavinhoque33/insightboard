//! Contract enforcement (D15, M4 decision 11): handler output is validated against
//! `contracts/api-contract.v1.json` with the `jsonschema` crate, formats included.

use serde_json::{json, Value};
use std::sync::LazyLock;

static CONTRACT: LazyLock<Value> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../../contracts/api-contract.v1.json"))
        .expect("contract must be valid JSON")
});

/// A validator for `root` (e.g. `{"$ref": "#/$defs/User"}`) with the contract's `$defs`.
fn validator(root: Value) -> jsonschema::Validator {
    let mut schema = root;
    let object = schema
        .as_object_mut()
        .expect("root schema must be an object");
    object.insert("$schema".into(), CONTRACT["$schema"].clone());
    object.insert("$defs".into(), CONTRACT["$defs"].clone());
    jsonschema::options()
        .should_validate_formats(true)
        .build(&schema)
        .expect("contract schema must compile")
}

fn violations(root: Value, instance: &Value) -> Vec<String> {
    validator(root)
        .iter_errors(instance)
        .map(|e| format!("{} at {}", e, e.instance_path()))
        .collect()
}

/// Panic unless `instance` satisfies `$defs/<def>`.
pub fn assert_def(def: &str, instance: &Value) {
    let errors = violations(json!({ "$ref": format!("#/$defs/{def}") }), instance);
    assert!(
        errors.is_empty(),
        "{def} contract violated: {errors:?}\n{instance:#}"
    );
}

/// Panic unless `instance` is an array whose items satisfy `$defs/<def>`.
pub fn assert_list_of(def: &str, instance: &Value) {
    let root = json!({ "type": "array", "items": { "$ref": format!("#/$defs/{def}") } });
    let errors = violations(root, instance);
    assert!(
        errors.is_empty(),
        "[{def}] contract violated: {errors:?}\n{instance:#}"
    );
}

/// Every error body: `ErrorResponse`, or `ValidationErrorResponse` for 422.
pub fn assert_error_body(status: axum::http::StatusCode, body: &Value) {
    if status == axum::http::StatusCode::UNPROCESSABLE_ENTITY {
        assert_def("ValidationErrorResponse", body);
    } else {
        assert_def("ErrorResponse", body);
    }
}

#[test]
fn whole_contract_compiles_as_a_schema() {
    let _ = validator(json!({ "$ref": "#/$defs/Dashboard" }));
    jsonschema::options()
        .build(&CONTRACT)
        .expect("the contract document itself must compile");
}

/// The deliberately broken fixture (drill 11): each mutation of a valid payload must fail
/// validation, and the unmodified payload must pass. If the validator stopped checking, this
/// test would fail.
#[test]
fn broken_fixtures_fail_validation() {
    let good = json!({
        "id": "8b0f6d9e-6a4f-4c6b-9d1e-1f2a3b4c5d6e",
        "user_id": "1c2d3e4f-5a6b-4c7d-8e9f-0a1b2c3d4e5f",
        "name": "Home",
        "layout_json": {"widgets": [{"id": "w1", "type": "weather", "x": 0, "y": 0, "w": 2, "h": 2}]},
        "settings_json": {},
        "created_at": "2026-10-10T12:00:00Z",
        "updated_at": "2026-10-10T12:00:00Z"
    });
    let root = json!({ "$ref": "#/$defs/Dashboard" });
    assert!(violations(root.clone(), &good).is_empty());

    let mut broken = Vec::new();
    let mut b = good.clone();
    b["layout_json"] = json!([]); // the pre-M4b default (D8)
    broken.push(b);
    let mut b = good.clone();
    b["layout_json"]["widgets"][0]["w"] = json!(0);
    broken.push(b);
    let mut b = good.clone();
    b["settings_json"] = Value::Null;
    broken.push(b);
    let mut b = good.clone();
    b["id"] = json!("not-a-uuid");
    broken.push(b);
    let mut b = good.clone();
    b["name"] = json!("   ");
    broken.push(b);
    let mut b = good.clone();
    b.as_object_mut().unwrap().remove("updated_at");
    broken.push(b);
    for fixture in &broken {
        assert!(
            !violations(root.clone(), fixture).is_empty(),
            "broken fixture passed validation: {fixture}"
        );
    }

    // Error bodies: 422 needs details; plain errors allow nothing else.
    let errors = json!({ "$ref": "#/$defs/ValidationErrorResponse" });
    assert!(!violations(errors.clone(), &json!({"error": "Validation failed"})).is_empty());
    assert!(!violations(
        errors,
        &json!({"error": "x", "details": [{"field": "email"}]})
    )
    .is_empty());
    assert!(!violations(
        json!({ "$ref": "#/$defs/ErrorResponse" }),
        &json!({"error": "x", "detail": "leak"})
    )
    .is_empty());

    // Status results: only the documented error classes.
    assert!(!violations(
        json!({ "$ref": "#/$defs/StatusCheck" }),
        &json!({"url": "http://x", "status": 0, "response_time_ms": 0, "error": "error sending request for url (http://x)"})
    )
    .is_empty());
}
