use crate::db_pc as db;
use crate::db_ws::get_work_after_id;
use crate::dto_pc::Work;

use rusqlite::Connection;

use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

struct Status {
    birth_time: u32,
    last_workitem_retrvd: i32,
    last_workitem_time_retrvd: u32,
    last_block_retrvd: u32,
    last_block_procd: u32,
    last_payment_procd: i32,
}

impl Status {
    fn new(birth_time: u32) -> Self { Self {
        birth_time,
        last_workitem_retrvd: -1,
        last_workitem_time_retrvd: 0,
        last_block_retrvd: 0,
        last_block_procd: 0,
        last_payment_procd: -1,
    }}
}

fn get_status_status(conn: &Connection, status: &mut Status) -> Result<(), Box<dyn Error>> {
    let (last_workitem_retrvd, last_block_retrvd, last_block_procd, last_payment_procd, last_workitem_time_retrvd) = db::get_status(conn)?;
    println!("Workitems: last retrieved: {last_workitem_retrvd} / {last_workitem_time_retrvd}");
    println!("Blocks: last retrieved / processed: {last_block_retrvd} / {last_block_procd}");
    status.last_workitem_retrvd = last_workitem_retrvd;
    status.last_workitem_time_retrvd = last_workitem_time_retrvd;
    status.last_block_retrvd = last_block_retrvd;
    status.last_block_procd = last_block_procd;
    status.last_payment_procd = last_payment_procd;
    Ok(())
}

fn print_status(status: &Status) {
    let now_utc = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    println!("Status: now {} \t last work ID {} / {} \t last block time {} \t last_payment_procd {}",
        now_utc, status.last_workitem_retrvd, status.last_workitem_time_retrvd, status.last_block_retrvd, status.last_payment_procd);
}

/// Get new workitems (from workitem.db)
/// Writes into paycalc, commits. Also updates last_workitem_retrvd & last_workitem_time_retrvd.
fn retrieve_new_workitems(conn_workitem_ro: &Connection, conn: &mut Connection, status: &mut Status, affected_user_ids: &mut HashSet<u32>) -> Result<usize, Box<dyn Error>> {
    println!("Getting new workitems, after ID {} / {}...", status.last_workitem_retrvd, status.last_workitem_time_retrvd);
    let start_time = std::cmp::max(status.birth_time, status.last_workitem_time_retrvd);

    let newworkitems = get_work_after_id(conn_workitem_ro, status.last_workitem_retrvd, start_time, 0)?;

    // # url = f"{workstat_api_url}/get-work-after-id?start_id={last_workitem_retrvd}&start_time={start_time}"
    // # response = requests.get(url)
    // # newworkitems = []
    // # if response.status_code != 200:
    // #     print(f"ERROR: Could not obtain new work items! url '{url}' {response}")
    // #     return 0
    // # newworkitems = response.json()
    // # # print(newworkitems)
    // # if not isinstance(newworkitems, list):
    // #     print(f"ERROR: Retrieved work items is not a list '{url}' {newworkitems}")
    // #     return 0
    // # # print(len(newworkitems))

    let cnt = newworkitems.len();
    // println!("cnt {}", cnt);
    if cnt == 0 {
        // No new workitems
        // println!(" ... none found");
        return Ok(0);
    }

    // Save them
    let new_last = newworkitems[cnt - 1].db_id;
    let new_last_time = newworkitems[cnt - 1].time_add.floor() as u32;

    let conntx = conn.transaction()?;
    for wi in newworkitems {
        let wi_pc = Work::new(
            wi.db_id,
            wi.uname_o, wi.uname_o_wrkr, wi.uname_u, wi.uname_u_wrkr,
            0, 0, 0, 0,
            wi.tdiff, wi.time_add, 0, 0, "".to_string(), 0, 0, 0, 0, 0);
        let wi_uname_o_id = wi_pc.uname_o_id;
        db::insert_work_struct_nocommit(&conntx, wi_pc)?;
        let _ = affected_user_ids.insert(wi_uname_o_id);
    }

    status.last_workitem_retrvd = new_last as i32;
    status.last_workitem_time_retrvd = new_last_time;
    let _ = db::set_status_last_workitem_retrvd(&conntx, new_last as i32, new_last_time)?;

    let _ = conntx.commit()?;
    get_status_status(conn, status)?;

    println!(" ... retrieved {} work records, last id {} / {}", cnt, status.last_workitem_retrvd, status.last_workitem_time_retrvd);
    Ok(cnt)
}

fn iteration(conn: &mut Connection, conn_workstat_ro: &Connection, _conn_oceanmgr_ro: &Connection, status: &mut Status)  -> Result<(), Box<dyn Error>> {
    // global last_block_procd
    // global birth_time
    get_status_status(conn, status)?;
    print_status(&status);

    let mut affected_user_ids = HashSet::<u32>::new();
    let cnt_wi = retrieve_new_workitems(conn_workstat_ro, conn, status, &mut affected_user_ids)?;

    // TODO TODO

    // cnt_bl1 = count_new_blocks(conn_oceanmgr_ro)
    // # print(f"New block count: {cnt_bl1}")

    // cnt_new_payment = retrieve_new_payments(conn, affected_user_ids)

    // if cnt_wi == 0 and cnt_bl1 == 0 and cnt_new_payment == 0:
    //     ## print(f"No new data found")
    //     return

    // cnt_bl2 = 0
    // new_blocks_accntd = 0
    // if cnt_wi > 0 or cnt_bl1 > 0:
    //     cnt_bl1 = retrieve_new_blocks(conn, conn_oceanmgr_ro)
    //     print_status()

    //     cursor = conn.cursor()
    //     tot_blocks_earned = db.block_get_total_earned(cursor)
    //     tot_work_comm_pre = db.work_get_total_committed(cursor)
    //     cursor.close()

    //     [cnt_bl2, new_blocks_accntd] = process_new_blocks(conn, affected_user_ids)
    //     print_status()

    //     # Blocks processed (zero or more)
    //     cursor = conn.cursor()
    //     tot_work_comm_post = db.work_get_total_committed(cursor)
    //     tot_work_estim_pre = db.work_get_total_estimated(cursor)
    //     cursor.close()

    //     # TODO fix
    //     expected_new_comm_msat = tot_blocks_earned * 1000
    //     if expected_new_comm_msat != tot_work_comm_post:
    //         print(f"ERROR: Total work committed and blocks committed mismatch {tot_work_comm_post} vs. {expected_new_comm_msat} diff {tot_work_comm_post - expected_new_comm_msat}   {tot_work_comm_pre} {tot_blocks_earned} {new_blocks_accntd}")

    //     # if cnt_bl2 == 0 and cnt_new_payment == 0:
    //     #     print(f"iteration, but no new block and no new payment ({last_block_procd}, earned {tot_blocks_earned}) ({last_payment_procd})")
    //     #     return

    // # Some info changed, update snapshots
    // avg_earn = get_avg_block_earn(conn)

    // cnt_considered = update_work_estimates(conn, birth_time, avg_earn, affected_user_ids)

    // cursor = conn.cursor()
    // tot_work_estim_post = db.work_get_total_estimated(cursor)
    // cursor.close()

    // create_new_miner_records_if_needed(conn, affected_user_ids)

    // if cnt_bl2 > 0:
    //     print(f"iteration: processed {cnt_bl2} block(s) with {new_blocks_accntd} msat, {last_block_procd}  {tot_blocks_earned}  comm {tot_work_comm_pre} -> {tot_work_comm_post} ({tot_work_comm_post - tot_work_comm_pre})  estim {tot_work_estim_pre} -> {tot_work_estim_post} ({tot_work_estim_post - tot_work_estim_pre})")

    // print(f"  new payments: {cnt_new_payment} ({last_payment_procd})")
    // print(f"  users affected: {len(affected_user_ids)}  workitems considered: {cnt_considered}")

    // # if len(affected_user_ids.keys()) >= 1:
    // #     id = list(affected_user_ids.keys())[0]
    // #     cursor = conn.cursor()
    // #     print(f"    miner user id {id} {db.userlookup_get_string(cursor, id)} {db.work_get_user_total_committed(cursor, id)} {db.work_get_user_total_estimated(cursor, id)}")
    // #     cursor.close()

    Ok(())
}

pub fn loop_iterations(conn: &mut Connection, conn_workstat_ro: &Connection, conn_oceanmgr_ro: &Connection) {
    // global birth_time
    let birth_time = env::var("PAYCALC_BIRTH_TIME").unwrap_or("0".to_string()).parse::<u32>().unwrap_or(0);
    let mut status = Status::new(birth_time);

    println!("Paycalc/main: loop starting...");
    if birth_time > 0 {
        println!("Birth time: {birth_time}");
    }

    let sleep_secs: f64 = 5.0;
    let mut next_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs_f64();

    loop {
        match iteration(conn, conn_workstat_ro, conn_oceanmgr_ro, &mut status) {
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

