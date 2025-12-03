mod db_ws;
mod dto_ws;

use crate::db_ws::{get_work_count, get_work_after_id};

use rusqlite::Connection;
use std::env;
use std::error::Error;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>>{
// Load environment variables from .env file
    dotenv::dotenv().ok();

    // Read DB_DIR from environment variables
    let db_dir = env::var("DB_DIR").unwrap_or_else(|_| "./data".to_string());
    println!("DB_DIR: {}", db_dir);

    let db_file = format!("{}/workstat.db", db_dir);
    println!("db_file: {}", db_file);

    let conn = Connection::open(&db_file)?;

    let count = get_work_count(&conn)?;
    println!("count: {}", count);

    let mut last_id = 0;
    let mut last_time: u32 = 0;
    loop {
        let workitems = get_work_after_id(&conn, last_id, last_time, 0)?;
        if workitems.len() == 0 {
            println!(".");
            thread::sleep(Duration::from_secs(3));
        } else {
            let last_item = &workitems[workitems.len() - 1];
            last_id = last_item.db_id();
            last_time = last_item.time_add() as u32;
            println!("Got {} workitems,  last: {} {}", workitems.len(), last_id, last_time);
        }
    }
    Ok(())
}
