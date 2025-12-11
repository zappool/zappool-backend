use paycalc_rust::common::get_db_file;
// use paycalc_rust::db_ws::{get_work_count, get_work_after_id};
// use paycalc_rust::db_oc::{get_new_blocks, count_new_blocks};
use paycalc_rust::paycalc_earn::loop_iterations;

use rusqlite::{Connection, OpenFlags};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // // Load environment variables from .env file
    // dotenv::dotenv().ok();

    // // Read DB_DIR from environment variables
    // let db_dir = env::var("DB_DIR").unwrap_or_else(|_| "./data".to_string());
    // println!("DB_DIR: {}", db_dir);

    let dbfile_workstat = get_db_file("workstat.db", false);
    let conn_workstat_ro = Connection::open_with_flags(dbfile_workstat, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

    let dbfile_oceanmgr = get_db_file("ocean.db", false);
    let conn_oceanmgr_ro = Connection::open_with_flags(dbfile_oceanmgr, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

    let dbfile = get_db_file("paycalc.db", false);
    let conn = Connection::open(&dbfile)?;

    loop_iterations(&conn, &conn_workstat_ro, &conn_oceanmgr_ro);

    Ok(())
}
