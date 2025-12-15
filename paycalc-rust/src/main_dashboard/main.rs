use paycalc_rust::common_db::get_db_file;
use paycalc_rust::paycalc_earn::{Status, get_status_status, print_blocks, print_block_stats, print_status};
use paycalc_rust::paycalc_payreq::{print_miner_snapshots, print_updated_miner_snapshots};

use rusqlite::{Connection, OpenFlags};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let dbfile = get_db_file("paycalc.db", false);
    let conn = Connection::open_with_flags(dbfile, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

    let mut status = Status::new(0);
    let _ = get_status_status(&conn, &mut status)?;
    let _ = print_status(&status);

    let _ = print_blocks(&conn)?;

    let _ = print_block_stats(&conn)?;

    let _ = print_miner_snapshots(&conn)?;

    let _ = print_updated_miner_snapshots(&conn)?;

    // print_node_info()

    // print_pay_total_stats(conn)

    // print_pay_requests(conn)

    // print_last_payments(conn, 7)

    Ok(())
}
