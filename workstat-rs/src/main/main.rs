use workstat_rs::server::start_server;

use common_rs::common_db::get_db_file;
use dotenv;
use std::env;

fn create_app(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/ping", get(ping))
        .route("/api/work-insert", post(add_work))
        .route("/api/work-count", get(get_count))
        .route("/api/get-work-after-id", get(get_work_after_id_handler))
        .with_state(state)
}

struct ServerHandle {
    pub addr: SocketAddr,
    #[cfg(test)]
    shutdown_tx: oneshot::Sender<()>,
    task: tokio::task::JoinHandle<()>,
}

impl ServerHandle {
    #[cfg(test)]
    async fn stop(self) {
        let _ = self.shutdown_tx.send(());
        let _ = self.task.await;
    }

    async fn wait(self) {
        let _ = self.task.await;
    }
}

async fn start_server(port: u16, dbfile: String, api_secret: String) -> ServerHandle {
    let state = Arc::new(AppState { dbfile, api_secret });
    let app = create_app(state);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();
    let addr = listener.local_addr().unwrap();
    #[cfg(not(test))]
    let (_shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    #[cfg(test)]
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
        #[cfg(test)]
        shutdown_tx,
        task,
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let dbfile = get_db_file("workstat.db", false);
    println!("Using dbfile: '{dbfile}'");

    let api_secret = env::var("WORKSTAT_SECRET").unwrap_or_default();
    if api_secret.len() < 2 {
        println!("Error: WORKSTAT_SECRET is unset or too weak");
        std::process::exit(-1);
    }
    println!("Using Api secret, {}", api_secret.len());

    let handle = start_server(5004, dbfile, api_secret).await;
    println!("Listening on {}", handle.addr);
    // Keep the process, never exit
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(30)).await
    }
    // handle.wait().await;
}
