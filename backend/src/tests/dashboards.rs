use axum::http::{Method, StatusCode};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use super::{
    contract::{assert_def, assert_error_body, assert_list_of},
    insert_user, test_config, TestApp,
};
use crate::handlers::dashboard::MAX_DASHBOARDS_PER_USER;

async fn setup(pool: PgPool) -> (TestApp, String, Uuid) {
    let user = insert_user(&pool, "owner@example.com").await;
    let app = TestApp::new(test_config(), pool).await;
    let token = app.token(user);
    (app, token, user)
}

fn layout() -> Value {
    json!({"widgets": [
        {"id": "w1", "type": "crypto", "x": 0, "y": 0, "w": 3, "h": 2, "config": {"ids": "bitcoin"}}
    ]})
}

#[sqlx::test(migrations = "./migrations")]
async fn create_without_json_fields_stores_contract_defaults(pool: PgPool) {
    let (app, token, _) = setup(pool.clone()).await;
    let res = app
        .json(
            Method::POST,
            "/api/dashboards",
            Some(&token),
            json!({"name": " Home "}),
        )
        .await;
    assert_eq!(res.status, StatusCode::CREATED);
    assert_def("Dashboard", &res.body);
    assert_eq!(res.body["name"], "Home");
    assert_eq!(res.body["layout_json"], json!({"widgets": []}));
    assert_eq!(res.body["settings_json"], json!({}));

    let id: Uuid = res.body["id"].as_str().unwrap().parse().unwrap();
    let (layout, settings): (Value, Value) =
        sqlx::query_as("SELECT layout_json, settings_json FROM dashboards WHERE id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(layout, json!({"widgets": []}));
    assert_eq!(settings, json!({}));
}

#[sqlx::test(migrations = "./migrations")]
async fn crud_responses_match_the_contract(pool: PgPool) {
    let (app, token, _) = setup(pool).await;
    let created = app
        .json(
            Method::POST,
            "/api/dashboards",
            Some(&token),
            json!({"name": "Main", "layout_json": layout(), "settings_json": {"theme": "dark", "custom": 1}}),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED);
    assert_def("Dashboard", &created.body);
    let uri = format!("/api/dashboards/{}", created.body["id"].as_str().unwrap());

    let got = app.get(&uri, Some(&token)).await;
    assert_eq!(got.status, StatusCode::OK);
    assert_def("Dashboard", &got.body);

    let updated = app
        .json(
            Method::PUT,
            &uri,
            Some(&token),
            json!({"layout_json": {"widgets": []}}),
        )
        .await;
    assert_eq!(updated.status, StatusCode::OK);
    assert_def("Dashboard", &updated.body);
    assert_eq!(updated.body["name"], "Main");
    assert_eq!(updated.body["settings_json"]["theme"], "dark");

    let list = app.get("/api/dashboards", Some(&token)).await;
    assert_eq!(list.status, StatusCode::OK);
    assert_list_of("Dashboard", &list.body);
    assert_eq!(list.body.as_array().unwrap().len(), 1);

    let deleted = app
        .send(super::build(Method::DELETE, &uri, Some(&token), None))
        .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);
    let gone = app.get(&uri, Some(&token)).await;
    assert_eq!(gone.status, StatusCode::NOT_FOUND);
    assert_error_body(gone.status, &gone.body);
}

#[sqlx::test(migrations = "./migrations")]
async fn blank_name_and_invalid_shapes_are_422_with_dotted_fields(pool: PgPool) {
    let (app, token, _) = setup(pool).await;
    let created = app
        .json(
            Method::POST,
            "/api/dashboards",
            Some(&token),
            json!({"name": "A"}),
        )
        .await;
    let uri = format!("/api/dashboards/{}", created.body["id"].as_str().unwrap());

    let res = app
        .json(Method::PUT, &uri, Some(&token), json!({"name": "   "}))
        .await;
    assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_error_body(res.status, &res.body);
    assert_eq!(res.body["details"][0]["field"], "name");

    let res = app
        .json(
            Method::PUT,
            &uri,
            Some(&token),
            json!({"layout_json": {"widgets": [{"id": "w", "type": "weather", "x": 0, "y": 0, "w": 0, "h": 1}]}}),
        )
        .await;
    assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_error_body(res.status, &res.body);
    assert_eq!(res.body["details"][0]["field"], "layout_json.widgets.0.w");

    let res = app
        .json(
            Method::POST,
            "/api/dashboards",
            Some(&token),
            json!({"name": "B", "layout_json": [], "settings_json": {"theme": "neon"}}),
        )
        .await;
    assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_error_body(res.status, &res.body);

    // Nothing was changed by the rejected update.
    let unchanged = app.get(&uri, Some(&token)).await;
    assert_eq!(unchanged.body["name"], "A");
}

#[sqlx::test(migrations = "./migrations")]
async fn foreign_dashboards_are_404_for_every_method(pool: PgPool) {
    let (app, owner_token, _) = setup(pool.clone()).await;
    let intruder = insert_user(&pool, "intruder@example.com").await;
    let intruder_token = app.token(intruder);
    let created = app
        .json(
            Method::POST,
            "/api/dashboards",
            Some(&owner_token),
            json!({"name": "Private"}),
        )
        .await;
    let uri = format!("/api/dashboards/{}", created.body["id"].as_str().unwrap());

    let get = app.get(&uri, Some(&intruder_token)).await;
    let put = app
        .json(
            Method::PUT,
            &uri,
            Some(&intruder_token),
            json!({"name": "Mine"}),
        )
        .await;
    let delete = app
        .send(super::build(
            Method::DELETE,
            &uri,
            Some(&intruder_token),
            None,
        ))
        .await;
    for res in [get, put, delete] {
        assert_eq!(res.status, StatusCode::NOT_FOUND);
        assert_error_body(res.status, &res.body);
    }
    let list = app.get("/api/dashboards", Some(&intruder_token)).await;
    assert_eq!(list.body, json!([]));
    let still = app.get(&uri, Some(&owner_token)).await;
    assert_eq!(still.body["name"], "Private");

    let bad_id = app
        .get("/api/dashboards/not-a-uuid", Some(&owner_token))
        .await;
    assert_eq!(bad_id.status, StatusCode::NOT_FOUND);
    assert_error_body(bad_id.status, &bad_id.body);
}

#[sqlx::test(migrations = "./migrations")]
async fn creating_beyond_the_cap_is_409(pool: PgPool) {
    let (app, token, user) = setup(pool.clone()).await;
    sqlx::query(
        "INSERT INTO dashboards (user_id, name) SELECT $1, 'd' || g FROM generate_series(1, $2) g",
    )
    .bind(user)
    .bind(MAX_DASHBOARDS_PER_USER as i32)
    .execute(&pool)
    .await
    .unwrap();

    let res = app
        .json(
            Method::POST,
            "/api/dashboards",
            Some(&token),
            json!({"name": "one too many"}),
        )
        .await;
    assert_eq!(res.status, StatusCode::CONFLICT);
    assert_error_body(res.status, &res.body);

    // The DB default from migration 003 is the contract shape.
    let list = app.get("/api/dashboards", Some(&token)).await;
    assert_list_of("Dashboard", &list.body);
    assert_eq!(
        list.body.as_array().unwrap().len(),
        MAX_DASHBOARDS_PER_USER as usize
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn create_for_a_deleted_user_is_401(pool: PgPool) {
    let app = TestApp::new(test_config(), pool).await;
    let token = app.token(Uuid::new_v4());
    let res = app
        .json(
            Method::POST,
            "/api/dashboards",
            Some(&token),
            json!({"name": "x"}),
        )
        .await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
}

/// Migration 003 backfills pre-M4b `null` / `[]` values.
#[sqlx::test(migrations = false)]
async fn migration_003_backfills_legacy_json(pool: PgPool) {
    use sqlx::migrate::Migrator;
    use std::path::Path;
    let all = Migrator::new(Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/migrations"
    )))
    .await
    .unwrap();
    let mut first_two = Migrator::new(Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/migrations"
    )))
    .await
    .unwrap();
    first_two.migrations = all
        .migrations
        .iter()
        .filter(|m| m.version < 3)
        .cloned()
        .collect::<Vec<_>>()
        .into();
    first_two.run(&pool).await.unwrap();

    let user = insert_user(&pool, "Legacy@Example.com ").await;
    for (layout, settings) in [("null", "null"), ("[]", "{}")] {
        sqlx::query(
            "INSERT INTO dashboards (user_id, name, layout_json, settings_json)
             VALUES ($1, 'legacy', $2::jsonb, $3::jsonb)",
        )
        .bind(user)
        .bind(layout)
        .bind(settings)
        .execute(&pool)
        .await
        .unwrap();
    }

    all.run(&pool).await.unwrap();

    let rows: Vec<(Value, Value)> =
        sqlx::query_as("SELECT layout_json, settings_json FROM dashboards")
            .fetch_all(&pool)
            .await
            .unwrap();
    for (layout, settings) in rows {
        assert_eq!(layout, json!({"widgets": []}));
        assert_eq!(settings, json!({}));
    }
    let (email,): (String,) = sqlx::query_as("SELECT email FROM users WHERE id = $1")
        .bind(user)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(email, "legacy@example.com");
}
