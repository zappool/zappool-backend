use paycalc_rust::common_db::get_db_file;
use paycalc_rust::paycalc_earn::loop_iterations;
use paycalc_rust::paycalc_payreq::loop_iterations as payreq_loop_iterations;

use dotenv;
use rusqlite::{Connection, OpenFlags};
use std::error::Error;
use std::thread;

fn main() -> Result<(), Box<dyn Error>> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // // Read DB_DIR from environment variables
    // let db_dir = env::var("DB_DIR").unwrap_or_else(|_| "./data".to_string());
    // println!("DB_DIR: {}", db_dir);

    let dbfile_workstat = get_db_file("workstat.db", false);
    let conn_workstat_ro =
        Connection::open_with_flags(dbfile_workstat, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

    let dbfile_oceanmgr = get_db_file("ocean.db", false);
    let conn_oceanmgr_ro =
        Connection::open_with_flags(dbfile_oceanmgr, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

    let dbfile = get_db_file("paycalc.db", false);
    let mut conn = Connection::open(&dbfile)?;

    // Start Payreq loop in background
    thread::spawn(|| match payreq_loop_iterations() {
        Err(e) => println!("Error: {e}"),
        Ok(_) => {}
    });

    // TODO
    // # Start payment loop in background
    // payer_thread = Thread(target=payer_loop_iterations, name="payer")
    // payer_thread.start()

    match loop_iterations(&mut conn, &conn_workstat_ro, &conn_oceanmgr_ro) {
        Err(e) => println!("Error: {e}"),
        Ok(_) => {}
    }

    Ok(())
}
