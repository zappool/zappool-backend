use common_rs::common::shorten_id;
use common_rs::db_ws;

use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
};
use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::oneshot;

const DEFAULT_POOL: u8 = 1;
const VALID_POOLS: [u8; 1] = [DEFAULT_POOL];

#[derive(Clone, Debug)]
struct AppState {
    dbfile: String,
    api_secret: String,
}

fn get_db_connection(dbfile: &str, readonly: bool) -> Result<Connection, rusqlite::Error> {
    if readonly {
        Connection::open_with_flags(dbfile, OpenFlags::SQLITE_OPEN_READ_ONLY)
    } else {
        Connection::open(dbfile)
    }
}

async fn ping() -> (StatusCode, Json<Value>) {
    println!("Received ping");
    (StatusCode::OK, Json(json!({"pong": "ok"})))
}

#[derive(Deserialize)]
pub struct WorkInsertRequest {
    uname_o: Option<String>,
    uname_u: Option<String>,
    tdiff: Option<Value>,
    sec: Option<String>,
    pool: Option<Value>,
}

async fn add_work(
    State(state): State<Arc<AppState>>,
    Json(data): Json<WorkInsertRequest>,
) -> (StatusCode, Json<Value>) {
    let uname_o = match data.uname_o {
        Some(v) => v,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Missing required field: uname_o"})),
            );
        }
    };
    let uname_u = match data.uname_u {
        Some(v) => v,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Missing required field: uname_u"})),
            );
        }
    };
    let tdiff_raw = match data.tdiff {
        Some(v) => v,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Missing required field: tdiff"})),
            );
        }
    };
    let tdiff: u32 = match tdiff_raw
        .as_u64()
        .or_else(|| tdiff_raw.as_str().and_then(|s| s.parse().ok()))
    {
        Some(v) if v > 0 => v as u32,
        Some(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Target difficulty must be a positive integer"})),
            );
        }
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Target difficulty must be an integer"})),
            );
        }
    };
    let secret = match data.sec {
        Some(v) => v,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Missing required field: sec"})),
            );
        }
    };
    let pool = match data.pool {
        Some(v) => {
            match v
                .as_u64()
                .or_else(|| tdiff_raw.as_str().and_then(|s| s.parse().ok()))
            {
                Some(v) => {
                    if v <= 255 {
                        let v8 = v as u8;
                        if VALID_POOLS.contains(&v8) {
                            v8
                        } else {
                            return (
                                StatusCode::BAD_REQUEST,
                                Json(json!({"error": format!("Unknown pool {v8}")})),
                            );
                        }
                    } else {
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(json!({"error": "Pool must be a byte"})),
                        );
                    }
                }
                None => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(json!({"error": "Pool must be an integer"})),
                    );
                }
            }
        }
        // If pool is missing, use the default
        None => DEFAULT_POOL,
    };

    // println!("Received work: '{}' '{}' {} P{}", shorten_id(&uname_o), shorten_id(&uname_u), tdiff, pool);

    if secret != state.api_secret {
        println!("Wrong API secret received!");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Incorrect API secret!"})),
        );
    }

    let conn = match get_db_connection(&state.dbfile, false) {
        Ok(c) => c,
        Err(e) => {
            println!("DB connection error: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("DB connection error: {e}")})),
            );
        }
    };

    match db_ws::insert_work_fullname(&conn, &uname_o, &uname_u, tdiff, pool) {
        Err(e) => {
            println!("Error inserting: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Error inserting {e}")})),
            )
        }
        Ok(_) => {
            println!(
                "Inserted work: '{}' '{}' {} P{}",
                shorten_id(&uname_o),
                shorten_id(&uname_u),
                tdiff,
                pool,
            );
            (
                StatusCode::CREATED,
                Json(json!({"message": "Work item added successfully"})),
            )
        }
    }
}

async fn get_count(State(state): State<Arc<AppState>>) -> (StatusCode, Json<Value>) {
    println!("Received get-count");

    let conn = match get_db_connection(&state.dbfile, true) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            );
        }
    };

    match db_ws::get_work_count(&conn) {
        Ok(cnt) => (StatusCode::OK, Json(json!({"work_count": cnt}))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

#[derive(Deserialize)]
struct GetWorkAfterIdParams {
    start_id: Option<String>,
    start_time: Option<String>,
    limit: Option<String>,
}

#[derive(Serialize)]
struct WorkJson {
    db_id: u32,
    uname_o: String,
    uname_o_wrkr: String,
    uname_u: String,
    uname_u_wrkr: String,
    tdiff: u32,
    pool: u8,
    time_add: f64,
    time_calc: u32,
    calc_payout: u32,
}

async fn get_work_after_id_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<GetWorkAfterIdParams>,
) -> (StatusCode, Json<Value>) {
    let start_id: i32 = match params.start_id.as_deref().and_then(|s| s.parse().ok()) {
        Some(v) if v != 0 => v,
        Some(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Invalid 'start_id' parameter!"})),
            );
        }
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Missing numeric 'start_id' parameter!"})),
            );
        }
    };
    let start_time: u32 = match params.start_time.as_deref().and_then(|s| s.parse().ok()) {
        Some(v) if v != 0 => v,
        Some(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Invalid 'start_time' parameter!"})),
            );
        }
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Missing numeric 'start_time' parameter!"})),
            );
        }
    };
    let limit: u32 = params
        .limit
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let conn = match get_db_connection(&state.dbfile, true) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            );
        }
    };

    match db_ws::get_work_after_id(&conn, start_id, start_time, limit) {
        Ok(work_list) => {
            let as_json: Vec<WorkJson> = work_list
                .into_iter()
                .map(|w| WorkJson {
                    db_id: w.db_id,
                    uname_o: w.uname_o,
                    uname_o_wrkr: w.uname_o_wrkr,
                    uname_u: w.uname_u,
                    uname_u_wrkr: w.uname_u_wrkr,
                    tdiff: w.tdiff,
                    pool: w.pool,
                    time_add: w.time_add,
                    time_calc: w.time_calc,
                    calc_payout: w.calc_payout,
                })
                .collect();
            (StatusCode::OK, Json(json!(as_json)))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

fn create_app(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/ping", get(ping))
        .route("/api/work-insert", post(add_work))
        .route("/api/work-count", get(get_count))
        .route("/api/get-work-after-id", get(get_work_after_id_handler))
        .with_state(state)
}

pub struct ServerHandle {
    pub addr: SocketAddr,
    // #[cfg(test)]
    shutdown_tx: oneshot::Sender<()>,
    task: tokio::task::JoinHandle<()>,
}

impl ServerHandle {
    // #[cfg(test)]
    pub async fn stop(self) {
        let _ = self.shutdown_tx.send(());
        let _ = self.task.await;
    }

    pub async fn wait(self) {
        let _ = self.task.await;
    }
}

pub async fn start_server(port: u16, dbfile: String, api_secret: String) -> ServerHandle {
    let state = Arc::new(AppState { dbfile, api_secret });
    let app = create_app(state);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();
    let addr = listener.local_addr().unwrap();
    // #[cfg(not(test))]
    // let (_shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    // #[cfg(test)]
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let task = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                shutdown_rx.await.ok();
            })
            .await
            .unwrap();
    });
    ServerHandle {
        addr,
        // #[cfg(test)]
        shutdown_tx,
        task,
    }
}
