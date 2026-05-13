use common_rs::db_ws::db_setup_from_to;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use rusqlite::Connection;
use serde_json::{Value, json};
use std::sync::Arc;
use tempfile::NamedTempFile;
use tower::ServiceExt;

use super::{AppState, create_app};

const TEST_SECRET: &str = "test_secret_xyz";

fn make_test_state() -> (NamedTempFile, Arc<AppState>) {
    let tmpfile = NamedTempFile::new().unwrap();
    let path = tmpfile.path().to_str().unwrap().to_string();
    let conn = Connection::open(&path).unwrap();
    db_setup_from_to(&conn, Some(0), None).unwrap();
    drop(conn);
    let state = Arc::new(AppState {
        dbfile: path,
        api_secret: TEST_SECRET.to_string(),
    });
    (tmpfile, state)
}

#[tokio::test]
async fn test_ping() {
    let (_tmp, state) = make_test_state();
    let app = create_app(state);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/ping")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024).await.unwrap();
    let parsed: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(parsed["pong"], "ok");
}

#[tokio::test]
async fn test_work_count_empty() {
    let (_tmp, state) = make_test_state();
    let app = create_app(state);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/work-count")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024).await.unwrap();
    let parsed: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(parsed["work_count"], 0);
}

#[tokio::test]
async fn test_work_insert() {
    let (_tmp, state) = make_test_state();
    let app = create_app(state);
    let payload = json!({
        "uname_o": "user1.worker1",
        "uname_u": "upstream.worker2",
        "tdiff": 131072,
        "sec": TEST_SECRET,
        "pool": 1,
    });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/work-insert")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), 1024).await.unwrap();
    let parsed: Value = serde_json::from_slice(&body).unwrap();
    assert!(parsed["message"].to_string().contains("uccess"));
}

#[tokio::test]
async fn test_work_count_after_insert() {
    let (_tmp, state) = make_test_state();

    let count_before: Value = {
        let app = create_app(Arc::clone(&state));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/work-count")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = to_bytes(response.into_body(), 1024).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    };
    assert_eq!(count_before["work_count"], 0);

    let payload = json!({
        "uname_o": "user1",
        "uname_u": "upstream",
        "tdiff": 65536,
        "sec": TEST_SECRET,
        "pool": 1,
    });
    let insert_response = create_app(Arc::clone(&state))
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/work-insert")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(insert_response.status(), StatusCode::CREATED);

    let count_after: Value = {
        let app = create_app(Arc::clone(&state));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/work-count")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = to_bytes(response.into_body(), 1024).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    };
    assert_eq!(count_after["work_count"], 1);
}

#[tokio::test]
async fn test_work_insert_invalid_pool() {
    let (_tmp, state) = make_test_state();
    let app = create_app(state);
    let payload = json!({
        "uname_o": "user1.worker1",
        "uname_u": "upstream.worker2",
        "tdiff": 131072,
        "sec": TEST_SECRET,
        "pool": 66,
    });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/work-insert")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
