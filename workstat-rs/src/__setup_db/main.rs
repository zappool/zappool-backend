use common_rs::db_ws::db_setup_1;

use common_rs::common_db::get_db_file;
use rusqlite::Connection;
use std::io::stdin;

fn main() {
    let dbfile = get_db_file("workstat.db", true);

    println!("Initialize DB '{dbfile}'. Press Y to continue");
    let mut buffer = String::new();
    stdin().read_line(&mut buffer).unwrap();
    if buffer.trim().to_uppercase() != "Y" {
        println!("Aborting");
        std::process::exit(1);
    }

    let conn = Connection::open(&dbfile).unwrap();
    db_setup_1(&conn).unwrap();
    let _ = conn.close();

    println!("New empty DB created, don't forget to rename! {dbfile}");
}
