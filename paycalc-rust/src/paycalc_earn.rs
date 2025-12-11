use rusqlite::Connection;

use std::env;
use std::error::Error;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn iteration(_conn: &Connection, _conn_workstat_ro: &Connection, _conn_oceanmgr_ro: &Connection)  -> Result<(), Box<dyn Error>> {
    println!("TODO!!!!!");
    Ok(())
}

pub fn loop_iterations(conn: &Connection, conn_workstat_ro: &Connection, conn_oceanmgr_ro: &Connection) {
    // global birth_time
    let birth_time = env::var("PAYCALC_BIRTH_TIME").unwrap_or("0".to_string()).parse::<u32>().unwrap_or(0);

    println!("Paycalc/main: loop starting...");
    if birth_time > 0 {
        println!("Birth time: {birth_time}");
    }

    let sleep_secs: f64 = 5.0;
    let mut next_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs_f64();

    loop {
        match iteration(conn, conn_workstat_ro, conn_oceanmgr_ro) {
            Ok(_) => {}
            Err(err) => {
                println!("ERROR in iteration, {}", err.to_string());
                continue;
            }
        }

        next_time = next_time + sleep_secs;
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs_f64();
        let to_sleep = f64::max(next_time - now, 0.1);
        if to_sleep > 0.0 {
            // println!("Sleeping for {}  secs... (next_time {})", to_sleep, next_time);
            thread::sleep(Duration::from_secs_f64(to_sleep));
        }
    }
}

