mod db_oc;
mod db_ws;
mod dto_oc;
mod dto_ws;

use crate::db_ws::{get_work_count, get_work_after_id};
use crate::db_oc::{get_new_blocks, count_new_blocks};

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

    let db_file_ws = format!("{}/workstat.db", db_dir);
    let db_file_oc = format!("{}/ocean.db", db_dir);
    println!("db_file: {} {}", db_file_ws, db_file_oc);

    let conn_ws = Connection::open(&db_file_ws)?;
    let conn_oc = Connection::open(&db_file_oc)?;

    let count = get_work_count(&conn_ws)?;
    println!("count: {}", count);

    let mut last_work_id = 0;
    let mut last_work_time: u32 = 0;
    let mut last_block_time: u32 = 0;
    loop {
        let workitems = get_work_after_id(&conn_ws, last_work_id, last_work_time, 0)?;
        if workitems.len() > 0 {
            let last_item = &workitems[workitems.len() - 1];
            last_work_id = last_item.db_id();
            last_work_time = last_item.time_add() as u32;
            println!("Got {} workitems,  last: {} {}", workitems.len(), last_work_id, last_work_time);
        }

        let block_count = count_new_blocks(&conn_oc, last_block_time)?;
        if block_count > 0 {
            let blocks = get_new_blocks(&conn_oc, last_block_time)?;
            if blocks.len() > 0 {
                let last_block = &blocks[blocks.len() - 1];
                last_block_time = last_block.time;
                println!("Got {} blocks,  last: {}", blocks.len(), last_block_time);
            }
        }

        if workitems.len() == 0 && block_count == 0 {
            println!(".");
            thread::sleep(Duration::from_secs(3));
        }
    }
    Ok(())
}
