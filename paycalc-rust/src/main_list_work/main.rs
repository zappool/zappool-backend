use paycalc_rust::common::get_db_file;
// use paycalc_rust::db_ws::{get_work_count, get_work_after_id};
// use paycalc_rust::db_oc::{get_new_blocks, count_new_blocks};
use paycalc_rust::db_pc::get_status;

use rusqlite::{Connection, OpenFlags};
use std::error::Error;

fn get_and_print_status_status(conn: &Connection) {
    let (
        last_workitem_retrvd,
        last_block_retrvd,
        last_block_procd,
        _last_payment_procd,
        last_workitem_time_retrvd
    ) = get_status(conn).unwrap();
    println!("Workitems: last retrieved/processed: {last_workitem_retrvd} / {last_workitem_time_retrvd}");
    println!("Blocks: last retrieved/processed: {last_block_retrvd} / {last_block_procd}");
}

fn main() -> Result<(), Box<dyn Error>> {
    let dbfile = get_db_file("paycalc.db", false);
    let conn_ro = Connection::open_with_flags(dbfile, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

    get_and_print_status_status(&conn_ro);

    // TODO
    // print_workitem_status(conn_workstat_ro, last_workitem_retrvd)
    Ok(())
}
