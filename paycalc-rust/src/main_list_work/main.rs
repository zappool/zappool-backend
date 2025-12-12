use paycalc_rust::common::get_db_file;
use paycalc_rust::db_ws::{get_work_count, get_work_after_id};
use paycalc_rust::db_pc::get_status;

use rusqlite::{Connection, OpenFlags};
use std::error::Error;

/// Returns last_workitem_retrvd
fn get_and_print_status_status(conn: &Connection) -> i32{
    let (
        last_workitem_retrvd,
        last_block_retrvd,
        last_block_procd,
        _last_payment_procd,
        last_workitem_time_retrvd
    ) = get_status(conn).unwrap();
    println!("Workitems: last retrieved/processed: {last_workitem_retrvd} / {last_workitem_time_retrvd}");
    println!("Blocks: last retrieved/processed: {last_block_retrvd} / {last_block_procd}");
    last_workitem_retrvd
}

fn print_workitem_status(conn: &Connection, last_workitem_retrvd: i32) {
    let workitemcnt = get_work_count(conn).unwrap();
    println!("Workitem count: {workitemcnt}");

    let newworkitems = get_work_after_id(&conn, last_workitem_retrvd, 0, 0).unwrap();
    println!("Workitems after ID {} are: ({} pcs):", last_workitem_retrvd, newworkitems.len());
    for wi in newworkitems {
        println!("  {} {} {}{}", wi.db_id, wi.tdiff, wi.uname_o, wi.uname_o_wrkr);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let dbfile_workstat = get_db_file("workstat.db", false);
    let conn_workstat_ro = Connection::open_with_flags(dbfile_workstat, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

    let dbfile = get_db_file("paycalc.db", false);
    let conn_ro = Connection::open_with_flags(dbfile, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

    let last_workitem_retrvd = get_and_print_status_status(&conn_ro);

    print_workitem_status(&conn_workstat_ro, last_workitem_retrvd);

    Ok(())
}
