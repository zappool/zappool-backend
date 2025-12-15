use paycalc_rust::common_db::{
    get_db_file, get_db_update_versions_from_args, print_current_db_version,
};
use paycalc_rust::db_pc::{LATEST_DB_VERSION, db_setup};

use rusqlite::Connection;
use std::io::stdin;

fn main() {
    let (vto, vfrom) = get_db_update_versions_from_args(LATEST_DB_VERSION);
    let create_mode: bool = if vfrom == 0 { true } else { false };
    let dbfile = get_db_file("paycalc.db", create_mode);

    print_current_db_version(&dbfile);

    println!("Create/Update DB v{vfrom} -> v{vto} '{dbfile}'. Press Y to continue");
    let mut buffer = String::new();
    stdin().read_line(&mut buffer).unwrap();
    let lineread = buffer.trim_end().to_uppercase().to_string();
    if lineread != "Y" {
        println!("Aborting");
        std::process::exit(-1);
    }
    // OK, continue
    // Connect to SQLite database
    let conn = Connection::open(&dbfile).unwrap();
    db_setup(&conn).unwrap();
    let _ = conn.close();

    print_current_db_version(&dbfile);
    println!("DB created/updated.  Check location!  {dbfile}");
}
