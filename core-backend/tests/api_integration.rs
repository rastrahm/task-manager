//! Pruebas HTTP de extremo a extremo contra PostgreSQL (`tasks_db_test`).
//!
//! ```bash
//! createdb tasks_db_test
//! psql -d tasks_db_test -f init.sql
//! cargo test -p task-core --features test-utils
//! ```

use axum::http::StatusCode;
use serde_json::json;
use task_core::test_support::{
    admin_password, bearer_from_auth, delete_request, get_json, login, patch_json, post_json,
    put_json, test_app,
};

#[tokio::test]
async fn login_succeeds_for_admin() {
    let app = test_app().await;
    let auth = login(&app, "admin", admin_password()).await;

    assert_eq!(auth["user"]["username"], "admin");
    assert_eq!(auth["user"]["is_admin"], true);
    assert_eq!(auth["token_type"], "Bearer");
    assert!(auth["access_token"].as_str().unwrap().len() > 20);
}

#[tokio::test]
async fn login_rejects_wrong_password() {
    let app = test_app().await;
    let (status, _) = post_json(
        &app,
        "/auth/login",
        json!({ "username": "admin", "password": "wrong" }),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn protected_route_requires_bearer_token() {
    let app = test_app().await;
    let (status, _) = get_json(&app, "/tasks", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn task_crud_and_tree() {
    let app = test_app().await;
    let auth = login(&app, "admin", admin_password()).await;
    let bearer = bearer_from_auth(&auth);

    let (status, created) = post_json(
        &app,
        "/tasks",
        json!({ "title": "Raíz", "metadata": {} }),
        Some(&bearer),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let root_id = created["id"].as_i64().unwrap();

    let (status, child) = post_json(
        &app,
        "/tasks",
        json!({ "title": "Hija", "parent_id": root_id, "metadata": {} }),
        Some(&bearer),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let child_id = child["id"].as_i64().unwrap();

    let (status, tree) = get_json(&app, "/tasks", Some(&bearer)).await;
    assert_eq!(status, StatusCode::OK);
    let roots = tree.as_array().unwrap();
    let root = roots.iter().find(|t| t["id"] == root_id).unwrap();
    assert_eq!(root["children"].as_array().unwrap().len(), 1);
    assert_eq!(root["children"][0]["id"], child_id);

    let (status, toggled) = post_json(&app, &format!("/tasks/{child_id}/toggle"), json!({}), Some(&bearer)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(toggled, json!(true));

    let (status, updated) = put_json(
        &app,
        &format!("/tasks/{child_id}"),
        json!({
            "title": "Hija renombrada",
            "description": "desc",
            "completed": true,
            "metadata": { "k": 1 },
            "parent_id": root_id
        }),
        Some(&bearer),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["title"], "Hija renombrada");
    assert_eq!(updated["completed"], true);

    let (status, patched) = patch_json(
        &app,
        &format!("/tasks/{child_id}/description"),
        json!({ "description": "nueva desc" }),
        Some(&bearer),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(patched["description"], "nueva desc");
}

#[tokio::test]
async fn refresh_rotates_token() {
    let app = test_app().await;
    let auth = login(&app, "admin", admin_password()).await;
    let refresh = auth["refresh_token"].as_str().unwrap().to_string();

    let (status, renewed) = post_json(
        &app,
        "/auth/refresh",
        json!({ "refresh_token": refresh }),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let new_refresh = renewed["refresh_token"].as_str().unwrap();
    assert_ne!(new_refresh, refresh);

    let (status, _) = post_json(
        &app,
        "/auth/refresh",
        json!({ "refresh_token": refresh }),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn admin_can_manage_users() {
    let app = test_app().await;
    let auth = login(&app, "admin", admin_password()).await;
    let bearer = bearer_from_auth(&auth);

    let (status, created) = post_json(
        &app,
        "/users",
        json!({
            "username": "bob",
            "password": "bobpass",
            "is_admin": false,
            "is_active": true
        }),
        Some(&bearer),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let bob_id = created["id"].as_i64().unwrap();

    let (status, users) = get_json(&app, "/users", Some(&bearer)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(users.as_array().unwrap().iter().any(|u| u["username"] == "bob"));

    let bob_auth = login(&app, "bob", "bobpass").await;
    let bob_bearer = bearer_from_auth(&bob_auth);

    let (status, _) = get_json(&app, "/users", Some(&bob_bearer)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, profile) = get_json(&app, &format!("/users/{bob_id}"), Some(&bob_bearer)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(profile["username"], "bob");

    let (status, _) = delete_request(&app, &format!("/users/{bob_id}"), Some(&bearer)).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}
