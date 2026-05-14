use crate::server::{ServerHandle, start_server};
use common_rs::db_ws::db_setup_1;

use axum::http::StatusCode;
use rusqlite::Connection;
use serde_json::{Value, json};
use serial_test::serial;
use std::collections::HashMap;
use tempfile::NamedTempFile;

const PORT: u16 = 5004;
const TEST_SECRET: &str = "test_secret_xyz";

async fn make_test_state() -> (NamedTempFile, ServerHandle) {
    let tmpfile = NamedTempFile::new().unwrap();
    let path = tmpfile.path().to_str().unwrap().to_string();
    let conn = Connection::open(&path).unwrap();
    db_setup_1(&conn).unwrap();
    drop(conn);
    let server_handle = start_server(PORT, path.clone(), TEST_SECRET.into()).await;
    (tmpfile, server_handle)
}

#[tokio::test]
#[serial]
async fn test_ping() {
    let (_tmp, server_handle) = make_test_state().await;
    let response = reqwest::get(format!("http://localhost:{}/api/ping", PORT))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let parsed = response.json::<HashMap<String, String>>().await.unwrap();
    assert_eq!(parsed["pong"], "ok");
    server_handle.stop().await;
}

#[tokio::test]
#[serial]
async fn test_work_count_empty() {
    let (_tmp, server_handle) = make_test_state().await;
    let response = reqwest::get(format!("http://localhost:{}/api/work-count", PORT))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let parsed = response.json::<HashMap<String, Value>>().await.unwrap();
    assert_eq!(parsed["work_count"], Value::Number(0.into()));
    server_handle.stop().await;
}

#[tokio::test]
#[serial]
async fn test_work_insert() {
    let (_tmp, server_handle) = make_test_state().await;
    let payload = json!({
        "uname_o": "user1.worker1",
        "uname_u": "upstream.worker2",
        "tdiff": 131072,
        "sec": TEST_SECRET,
    });
    let response = reqwest::Client::new()
        .post(format!("http://localhost:{}/api/work-insert", PORT))
        .header("Content-Type", "application/json")
        .body(payload.to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let parsed = response.json::<HashMap<String, String>>().await.unwrap();
    assert!(parsed["message"].contains("uccess"));
    server_handle.stop().await;
}

#[tokio::test]
#[serial]
async fn test_work_count_after_insert() {
    let (_tmp, server_handle) = make_test_state().await;
    {
        let response = reqwest::get(format!("http://localhost:{}/api/work-count", PORT))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let parsed = response.json::<HashMap<String, Value>>().await.unwrap();
        assert_eq!(parsed["work_count"], Value::Number(0.into()));
    }
    {
        let payload = json!({
            "uname_o": "user1.worker1",
            "uname_u": "upstream.worker2",
            "tdiff": 131072,
            "sec": TEST_SECRET,
            "pool": 1,
        });
        let response = reqwest::Client::new()
            .post(format!("http://localhost:{}/api/work-insert", PORT))
            .header("Content-Type", "application/json")
            .body(payload.to_string())
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let parsed = response.json::<HashMap<String, String>>().await.unwrap();
        assert!(parsed["message"].contains("uccess"));
    }
    {
        let response = reqwest::get(format!("http://localhost:{}/api/work-count", PORT))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let parsed = response.json::<HashMap<String, Value>>().await.unwrap();
        assert_eq!(parsed["work_count"], Value::Number(1.into()));
    }
    server_handle.stop().await;
}
