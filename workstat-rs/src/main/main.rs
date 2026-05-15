use workstat_rs::server::start_server;

use common_rs::common_db::get_db_file;
use dotenv;
use std::env;

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
    // // Keep the process, never exit
    // loop {
    //     tokio::time::sleep(std::time::Duration::from_secs(30)).await
    // }
    handle.wait().await;
}
