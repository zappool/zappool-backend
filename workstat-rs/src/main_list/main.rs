use common_rs::db_ws::{get_all_work_limit, get_work_count};

use common_rs::common_db::get_db_file;
use rusqlite::{Connection, OpenFlags};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let dbfile = get_db_file("workstat.db", false);
    let conn = Connection::open_with_flags(dbfile, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

    let count = get_work_count(&conn)?;
    println!("Total work count: {count}");

    let work_list = get_all_work_limit(&conn, 100)?;
    println!("({}):", work_list.len());
    for w in work_list {
        println!(
            "{} {} {} {} {} {} {} {} {} {}",
            w.db_id,
            w.uname_o,
            w.uname_o_wrkr,
            w.uname_u,
            w.uname_u_wrkr,
            w.tdiff,
            w.pool,
            w.time_add,
            w.time_calc,
            w.calc_payout
        );
    }

    Ok(())
}
