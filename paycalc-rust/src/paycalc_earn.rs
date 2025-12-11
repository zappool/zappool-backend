use rusqlite::Connection;

use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::thread;

pub fn loop_iterations(_conn: &Connection, _conn_workstat_ro: &Connection, _conn_oceanmgr_ro: &Connection) {
    // TODO

    // global birth_time
    // birth_time = int(os.getenv("PAYCALC_BIRTH_TIME", 0))

    println!("Paycalc/main: loop starting...");
    // if birth_time > 0:
    //     print(f"Birth time: {birth_time}")

    let sleep_secs: f64 = 5.0;
    let mut next_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs_f64();

    loop {
        // try:
        //     iteration(conn, conn_workstat_ro, conn_oceanmgr_ro)
        // except Exception as ex:
        //     print(f"ERROR in iteration, {str(ex)}")
        //     continue

        next_time = next_time + sleep_secs;
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs_f64();
        let to_sleep = f64::max(next_time - now, 0.1);
        if to_sleep > 0.0 {
            // println!("Sleeping for {}  secs... (next_time {})", to_sleep, next_time);
            thread::sleep(Duration::from_secs_f64(to_sleep));
        }
    }
}

