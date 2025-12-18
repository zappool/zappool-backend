use paycalc_rs::cln_pay::print_node_info;
use paycalc_rs::common_db::get_db_file;
use paycalc_rs::paycalc_earn::{
    Status, get_status_status, print_block_stats, print_blocks, print_status,
};
use paycalc_rs::paycalc_payreq::{
    print_miner_snapshots, print_pay_requests, print_pay_total_stats, print_updated_miner_snapshots,
};
use paycalc_rs::payer::print_last_payments;

use rusqlite::{Connection, OpenFlags};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // paycalc_rs::ln_address::do_try().await;

    let dbfile = get_db_file("paycalc.db", false);
    let conn = Connection::open_with_flags(dbfile, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

    let mut status = Status::new(0);
    let _ = get_status_status(&conn, &mut status)?;
    let _ = print_status(&status);

    let _ = print_blocks(&conn)?;

    let _ = print_block_stats(&conn)?;

    let _ = print_miner_snapshots(&conn)?;

    let _ = print_updated_miner_snapshots(&conn)?;

    let _ = print_node_info().await.ok();

    let _ = print_pay_total_stats(&conn)?;

    let _ = print_pay_requests(&conn)?;

    let _ = print_last_payments(&conn, 7)?;

    Ok(())
}
