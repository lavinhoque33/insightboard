use axum::http::{Method, StatusCode};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use super::{
    contract::{assert_def, assert_error_body},
    test_config, TestApp,
};

const REGISTER: &str = "/api/auth/register";
const LOGIN: &str = "/api/auth/login";

async fn app(pool: PgPool) -> TestApp {
    TestApp::new(test_config(), pool).await
}

#[sqlx::test(migrations = "./migrations")]
async fn register_login_and_me_match_the_contract(pool: PgPool) {
    let app = app(pool).await;
    let res = app
        .json(
            Method::POST,
            REGISTER,
            None,
            json!({"email": "  New.User@Example.COM ", "password": "password123"}),
        )
        .await;
    assert_eq!(res.status, StatusCode::CREATED);
    assert_def("AuthResponse", &res.body);
    assert_eq!(res.body["user"]["email"], "new.user@example.com");

    let res = app
        .json(
            Method::POST,
            LOGIN,
            None,
            json!({"email": "NEW.USER@example.com", "password": "password123"}),
        )
        .await;
    assert_eq!(res.status, StatusCode::OK);
    assert_def("AuthResponse", &res.body);
    let token = res.body["token"].as_str().unwrap().to_string();

    let res = app.get("/api/me", Some(&token)).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_def("User", &res.body);
}

#[sqlx::test(migrations = "./migrations")]
async fn duplicate_email_is_409_even_with_different_case(pool: PgPool) {
    let app = app(pool).await;
    let body = json!({"email": "dup@example.com", "password": "password123"});
    let first = app.json(Method::POST, REGISTER, None, body).await;
    assert_eq!(first.status, StatusCode::CREATED);
    let second = app
        .json(
            Method::POST,
            REGISTER,
            None,
            json!({"email": " DUP@example.com", "password": "password456"}),
        )
        .await;
    assert_eq!(second.status, StatusCode::CONFLICT);
    assert_error_body(second.status, &second.body);
}

#[sqlx::test(migrations = "./migrations")]
async fn concurrent_duplicate_registrations_yield_one_201_and_409s(pool: PgPool) {
    let app = app(pool).await;
    let body = json!({"email": "race@example.com", "password": "password123"});
    let (a, b, c) = tokio::join!(
        app.json(Method::POST, REGISTER, None, body.clone()),
        app.json(Method::POST, REGISTER, None, body.clone()),
        app.json(Method::POST, REGISTER, None, body.clone()),
    );
    let mut statuses = vec![a.status, b.status, c.status];
    statuses.sort();
    assert_eq!(
        statuses,
        [
            StatusCode::CREATED,
            StatusCode::CONFLICT,
            StatusCode::CONFLICT
        ]
    );
    for res in [a, b, c] {
        if res.status == StatusCode::CONFLICT {
            assert_error_body(res.status, &res.body);
        }
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn invalid_register_input_is_422_with_details(pool: PgPool) {
    let app = app(pool).await;
    let res = app
        .json(
            Method::POST,
            REGISTER,
            None,
            json!({"email": "not-an-email", "password": "short"}),
        )
        .await;
    assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_error_body(res.status, &res.body);
    let fields: Vec<&str> = res.body["details"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["field"].as_str().unwrap())
        .collect();
    assert_eq!(fields, ["email", "password"]);

    let res = app
        .json(Method::POST, REGISTER, None, json!({"email": 7}))
        .await;
    assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_error_body(res.status, &res.body);
}

#[sqlx::test(migrations = "./migrations")]
async fn wrong_password_and_unknown_email_are_the_same_generic_401(pool: PgPool) {
    let app = app(pool).await;
    app.json(
        Method::POST,
        REGISTER,
        None,
        json!({"email": "known@example.com", "password": "password123"}),
    )
    .await;
    let wrong = app
        .json(
            Method::POST,
            LOGIN,
            None,
            json!({"email": "known@example.com", "password": "wrong-password"}),
        )
        .await;
    let unknown = app
        .json(
            Method::POST,
            LOGIN,
            None,
            json!({"email": "unknown@example.com", "password": "password123"}),
        )
        .await;
    for res in [&wrong, &unknown] {
        assert_eq!(res.status, StatusCode::UNAUTHORIZED);
        assert_eq!(res.body, json!({"error": "Unauthorized"}));
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn me_for_a_deleted_user_is_401(pool: PgPool) {
    let app = app(pool.clone()).await;
    let token = app.token(Uuid::new_v4());
    let res = app.get("/api/me", Some(&token)).await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    assert_eq!(res.body, json!({"error": "Unauthorized"}));
}

#[tokio::test]
async fn bad_tokens_get_the_generic_401() {
    let app = TestApp::without_db().await;
    for token in [None, Some("garbage"), Some("a.b.c")] {
        let res = app.get("/api/me", token).await;
        assert_eq!(res.status, StatusCode::UNAUTHORIZED);
        assert_eq!(res.body, json!({"error": "Unauthorized"}));
    }
    let foreign =
        crate::auth::generate_token(Uuid::new_v4(), "x@example.com", &"z".repeat(40)).unwrap();
    let res = app.get("/api/dashboards", Some(&foreign)).await;
    assert_eq!(res.body, json!({"error": "Unauthorized"}));
}
