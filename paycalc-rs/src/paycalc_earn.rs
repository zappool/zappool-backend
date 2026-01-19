use crate::db_oc;
use crate::db_ws::get_work_after_id;

use common_rs::db_pc as db;
use common_rs::dto_pc::{Block, MinerSnapshot, Work};

use rusqlite::Connection;

use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const BLOCKS_WINDOW: u16 = 8;
const BLOCK_AVERAGE_EARNING_COUNT: u32 = 16;

pub struct Status {
    birth_time: u32,
    last_workitem_retrvd: i32,
    last_workitem_time_retrvd: u32,
    last_block_retrvd: u32,
    last_block_procd: u32,
    last_payment_procd: i32,
}

impl Status {
    pub fn new(birth_time: u32) -> Self {
        Self {
            birth_time,
            last_workitem_retrvd: -1,
            last_workitem_time_retrvd: 0,
            last_block_retrvd: 0,
            last_block_procd: 0,
            last_payment_procd: -1,
        }
    }
}

pub fn get_status_status(conn: &Connection, status: &mut Status) -> Result<(), Box<dyn Error>> {
    let (
        last_workitem_retrvd,
        last_block_retrvd,
        last_block_procd,
        last_payment_procd,
        last_workitem_time_retrvd,
    ) = db::get_status(conn)?;
    // println!("Workitems: last retrieved: {last_workitem_retrvd} / {last_workitem_time_retrvd}");
    // println!("Blocks: last retrieved / processed: {last_block_retrvd} / {last_block_procd}");
    status.last_workitem_retrvd = last_workitem_retrvd;
    status.last_workitem_time_retrvd = last_workitem_time_retrvd;
    status.last_block_retrvd = last_block_retrvd;
    status.last_block_procd = last_block_procd;
    status.last_payment_procd = last_payment_procd;
    Ok(())
}

pub fn print_status(status: &Status) {
    let now_utc = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    println!(
        "Status: now {} \t last work ID {} / {} \t last block time {} \t last_payment_procd {}",
        now_utc,
        status.last_workitem_retrvd,
        status.last_workitem_time_retrvd,
        status.last_block_retrvd,
        status.last_payment_procd
    );
}

pub fn print_blocks(conn: &Connection) -> Result<(), Box<dyn Error>> {
    let days = 7;
    let now_utc = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as u32;
    let start_time = now_utc - days * 86400;
    let blocks = db::block_get_new_blocks(conn, start_time)?;
    println!();
    println!("Recent Blocks: ({} in past {} days)", blocks.len(), days);
    println!("  age (hr) \t earned \t tdiff \t poolfee \t unitearn(msat/128k)");
    for b in blocks {
        let age_hr = ((now_utc - b.time) as f64) / 3600.0;
        let mut unit_earn: f64 = 0.0;
        if b.acc_total_diff != 0 {
            unit_earn = (b.earned_sats) as f64 / (b.acc_total_diff) as f64 * 131072.0 * 1000.0;
        }
        println!(
            "  {:.1} \t {} \t {} \t {:.0} \t {:.2}",
            age_hr, b.earned_sats, b.acc_total_diff, b.pool_fee, unit_earn
        );
    }
    Ok(())
}

pub fn print_block_stats(conn: &Connection) -> Result<(), Box<dyn Error>> {
    println!();
    let total_earn = db::block_get_total_earn(conn).unwrap_or(0);
    println!("Total earnings:  {total_earn} sats");
    let count = BLOCK_AVERAGE_EARNING_COUNT;
    let (sum_earn, sum_diff) = db::block_get_last_avg_n(conn, count)?;
    let mut avg_earn = 0.0;
    if sum_diff > 0 {
        avg_earn = 1.0 / (sum_diff as f64) * (sum_earn as f64);
    }
    println!(
        "Avg earn from the {} last blocks (msats/128kdiff):  {:.1} ({} {})",
        count,
        avg_earn * 1000.0 * 131072.0,
        sum_earn,
        sum_diff
    );
    Ok(())
}

/// Get new workitems (from workitem.db)
/// Writes into paycalc, commits. Also updates last_workitem_retrvd & last_workitem_time_retrvd.
fn retrieve_new_workitems(
    conn_workitem_ro: &Connection,
    conn: &mut Connection,
    status: &mut Status,
    affected_user_ids: &mut HashSet<u32>,
) -> Result<usize, Box<dyn Error>> {
    // println!("Getting new workitems, after ID {} / {}...", status.last_workitem_retrvd, status.last_workitem_time_retrvd);
    let start_time = std::cmp::max(status.birth_time, status.last_workitem_time_retrvd);

    let newworkitems =
        get_work_after_id(conn_workitem_ro, status.last_workitem_retrvd, start_time, 0)?;

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
            wi.uname_o,
            wi.uname_o_wrkr,
            wi.uname_u,
            wi.uname_u_wrkr,
            0,
            0,
            0,
            0,
            wi.tdiff,
            wi.time_add,
            0,
            0,
            "".to_string(),
            0,
            0,
            0,
            0,
            0,
        );
        let (wi2, _cnt) = db::insert_work_struct_nocommit(&conntx, wi_pc)?;
        let _ = affected_user_ids.insert(wi2.uname_o_id);
    }

    status.last_workitem_retrvd = new_last as i32;
    status.last_workitem_time_retrvd = new_last_time;
    let _ = db::set_status_last_workitem_retrvd(&conntx, new_last as i32, new_last_time)?;

    let _ = conntx.commit()?;
    let _ = get_status_status(conn, status)?;

    println!(
        " ... retrieved {} work records, last id {} / {}",
        cnt, status.last_workitem_retrvd, status.last_workitem_time_retrvd
    );
    Ok(cnt)
}

/// Count new blocks (from ocean.db)
fn count_new_blocks(conn_oceanmgr_ro: &Connection, status: &Status) -> Result<u32, Box<dyn Error>> {
    let cutoff_time = std::cmp::max(status.last_block_retrvd, status.birth_time);
    let cnt = db_oc::count_new_blocks(conn_oceanmgr_ro, cutoff_time)?;
    // println!("New block count: {cnt}");
    Ok(cnt)
}

// Get new blocks (from ocean.db)
// Writes into paycalc, commits. Also updates last_block_retrvd.
fn retrieve_new_blocks(
    conn: &mut Connection,
    conn_oceanmgr_ro: &Connection,
    status: &mut Status,
) -> Result<u32, Box<dyn Error>> {
    // println!("Getting new blocks, after time {last_block_retrvd} ...");
    let cutoff_time = std::cmp::max(status.last_block_retrvd, status.birth_time);
    let new_blocks = db_oc::get_new_blocks(conn_oceanmgr_ro, cutoff_time)?;
    // last_block_time = 0
    // if len(new_blocks) >= 1:
    //     last_block_time = new_blocks[len(new_blocks) - 1].time
    // print(f"Retrieved {len(new_blocks)} blocks (last time {last_block_time})")

    let cnt = new_blocks.len();
    if cnt == 0 {
        // No new blocks
        // println!(" ... none found");
        return Ok(0);
    }

    // Save them
    let new_last = new_blocks[cnt - 1].time;
    let now_utc = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as u32;

    let conntx = conn.transaction()?;
    for bo in new_blocks {
        let bp = Block::new(bo.time, bo.block_hash, bo.earned_sats, bo.pool_fee, 0);
        let _ = db::block_insert(&conntx, &bp, now_utc)?;
    }

    let _ = db::set_status_last_block_retrvd(&conntx, new_last)?;

    let _ = conntx.commit()?;
    let _ = get_status_status(conn, status)?;

    println!(
        " ... retrieved {} blocks, last time {}",
        cnt, status.last_block_retrvd
    );
    Ok(cnt as u32)
}

// Account for block earnings
// Return:
// - total (new) committed earnings accounted (msat)
// - total diff of affected work items
fn account_for_new_block(
    conn: &mut Connection,
    block_time: u32,
    new_earnings: u64,
    status: &mut Status,
    affected_user_ids: &mut HashSet<u32>,
) -> Result<(u64, u64), Box<dyn Error>> {
    let tot_comm_pre = db::work_get_total_committed(conn)?;
    println!("Processing block {block_time}, {new_earnings}  {tot_comm_pre}");

    let work = db::work_get_affected_by_new_block(conn, block_time)?;

    if work.len() == 0 {
        println!("No workitems to update found!");
    }

    let mut total_diff = 0;
    for w in &work {
        total_diff += w.tdiff as u64;
    }
    println!(
        "Found {} affected workitems, new earn {}  total diff {}",
        work.len(),
        new_earnings,
        total_diff
    );

    let mut remain_earn_msat = new_earnings * 1000;
    let mut remain_diff = total_diff;

    let mut total_accounted = 0;
    let mut work_copy = Vec::new();
    for mut w in work {
        let diff1 = w.tdiff;
        //println!(f"{}", (1.0 / (remain_diff as f64) * (remain_earn_msat as f64) * (diff1 as f64)));
        let earn1_msat = (1.0 / (remain_diff as f64) * (remain_earn_msat as f64) * (diff1 as f64))
            .round() as u64;
        //println!(f"earn1_msat {earn1_msat}");
        w.committed += earn1_msat;
        total_accounted += earn1_msat;
        if w.commit_blocks < BLOCKS_WINDOW {
            w.commit_blocks += 1;
            if w.commit_blocks == 1 {
                w.commit_first_time = block_time;
            }
            if w.commit_blocks == BLOCKS_WINDOW {
                w.estimate = 0;
            }
        }
        w.commit_next_time = block_time;
        let w_uname_o_id = w.uname_o_id;
        work_copy.push(w);
        remain_earn_msat -= earn1_msat;
        remain_diff -= diff1 as u64;
        affected_user_ids.insert(w_uname_o_id);
    }

    // Update in DB
    let conntx = conn.transaction()?;
    for w in &work_copy {
        db::work_update_nocommit(&conntx, w)?;
    }

    // All have been updated
    status.last_block_procd = block_time;
    let _ = db::set_status_last_block_procd(&conntx, status.last_block_procd)?;
    //self.last_block_time = block_time

    let _ = db::block_update_diff_no_commit(&conntx, block_time, total_diff)?;

    let _ = conntx.commit()?;

    let _ = get_status_status(conn, status)?;

    Ok((total_accounted, total_diff))
}

// Account for a new block.
// Find all applicable work items, and update them
// Return total (new) committed earning accounted
fn process_new_block(
    conn: &mut Connection,
    nb: &Block,
    status: &mut Status,
    affected_user_ids: &mut HashSet<u32>,
) -> Result<u64, Box<dyn Error>> {
    if nb.earned_sats == 0 {
        println!("ERROR: Block has 0 earning!");
        return Ok(0);
    }
    if nb.time <= status.last_block_procd {
        println!("ERROR: Block already processed!");
        return Ok(0);
    }

    let tot_comm_pre = db::work_get_total_committed(conn)?;

    let (total_accounted, _total_diff) =
        account_for_new_block(conn, nb.time, nb.earned_sats, status, affected_user_ids)?;

    let tot_comm_post = db::work_get_total_committed(conn)?;

    println!(
        "Processed block  {}, {},  {} -> {} ({})",
        nb.time,
        nb.earned_sats,
        tot_comm_pre,
        tot_comm_post,
        tot_comm_post - tot_comm_pre
    );
    Ok(total_accounted)
}

// Check for new, unaccounted blocks, and account for them.
// Return:
// - the number of blocks processed
// - total (new) committed earning accounted
fn process_new_blocks(
    conn: &mut Connection,
    status: &mut Status,
    affected_user_ids: &mut HashSet<u32>,
) -> Result<(u32, u64), Box<dyn Error>> {
    let _ = get_status_status(conn, status)?;
    let new_blocks = db::block_get_new_blocks(conn, status.last_block_procd)?;
    if new_blocks.len() == 0 {
        println!(
            "No newer blocks found, last_block_procd {}",
            status.last_block_procd
        );
        return Ok((0, 0));
    }
    println!(
        "Last proc block {},  found {} newer blocks",
        status.last_block_procd,
        new_blocks.len()
    );

    let mut total_accounted = 0;
    for b in &new_blocks {
        // println!("New block time: {b.time}");
        let accntd1 = process_new_block(conn, &b, status, affected_user_ids)?;
        total_accounted += accntd1;
    }

    Ok((new_blocks.len() as u32, total_accounted))
}

// Return if new payments found
fn retrieve_new_payments(
    conn: &mut Connection,
    status: &mut Status,
    affected_user_ids: &mut HashSet<u32>,
) -> Result<u32, Box<dyn Error>> {
    let cutoff_time = std::cmp::max(status.last_payment_procd, status.birth_time as i32) as u32;
    let new_payments = db::payment_get_all_after_time(conn, cutoff_time)?;
    //print(f"new_payments {len(new_payments)}  {new_payments}")

    if new_payments.is_empty() {
        return Ok(0);
    }

    for (pr, _paym) in &new_payments {
        affected_user_ids.insert(pr.miner_id);
    }

    let last_time = new_payments[new_payments.len() - 1].1.status_time;
    //print(f"Updating last_payment_procd from {last_payment_procd} to {last_time}")
    status.last_payment_procd = last_time as i32;
    let conntx = conn.transaction()?;
    let _ = db::set_status_last_payment_procd(&conntx, status.last_payment_procd as u32)?;
    let _ = conntx.commit()?;
    let _ = get_status_status(conn, status)?;
    println!("New last_payment_procd: {}", status.last_payment_procd);
    Ok(new_payments.len() as u32)
}

// Update estimate for given workitems, if not fully committed
// Return the changed workitems
fn update_given_work_estimates(
    work: &Vec<Work>,
    avg_earn_per_diff_sat: f64,
) -> Result<Vec<Work>, Box<dyn Error>> {
    let now_utc = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as u32;
    let mut updated_work = Vec::new();
    let mut new_estimate;
    for w in work {
        if w.commit_blocks >= BLOCKS_WINDOW {
            new_estimate = 0;
        } else {
            let rem_blocks = BLOCKS_WINDOW - w.commit_blocks;
            // old_estimate = self.work[i].estimate
            new_estimate = ((rem_blocks as f64) * (w.tdiff as f64) * avg_earn_per_diff_sat * 1000.0)
                .round() as u64;
            //println!("  estimate {rem_blocks} {new_estimate} {w.estimate} {w.committed}");
        }
        if new_estimate != w.estimate {
            println!(
                "  new estimate:  {}  vs.  {}  {} {}  addage {}",
                new_estimate,
                w.estimate,
                w.commit_blocks,
                w.committed,
                now_utc as i32 - (w.time_add).round() as i32
            );
            let mut wcopy = w.clone();
            wcopy.estimate = new_estimate;
            updated_work.push(wcopy);
        }
    }
    println!(
        "update_given_work_estimates:  found {}/{} estimates to be updated",
        updated_work.len(),
        work.len()
    );
    Ok(updated_work)
}

// Update estimate for workitems not fully accounted
// Return number of workitems considered (updated or unchanged)
fn update_work_estimates(
    conn: &mut Connection,
    birth_time: u32,
    avg_earn_per_diff_sat: f64,
    affected_user_ids: &mut HashSet<u32>,
) -> Result<u32, Box<dyn Error>> {
    let work = db::work_get_for_estimate_update(conn, birth_time)?;
    // println!("{} workitems found for estimation", work.len());
    if work.is_empty() {
        return Ok(0);
    }
    let updated_work = update_given_work_estimates(&work, avg_earn_per_diff_sat)?;
    if !updated_work.is_empty() {
        // Save them
        let mut conntx = conn.transaction()?;
        for w in &updated_work {
            let _ = db::work_update_nocommit(&mut conntx, w)?;
            affected_user_ids.insert(w.uname_o_id);
        }
        let _ = conntx.commit()?;
    }
    println!(
        "Updated estimates: {}/{} workitem estimates updated",
        updated_work.len(),
        work.len()
    );
    Ok(work.len() as u32)
}

/*
# Update estimate for all workitems
# Return number of workitems updated
def update_all_work_estimates_nocommit(conn: sqlite3.Connection, birth_time: int, avg_earn_per_diff_sat: float) -> int:
    cursor = conn.cursor()
    work = db.work_get_all(cursor, birth_time)
    # print(f"{len(work)} workitems found for estimation")
    if len(work) == 0:
        return 0
    updated_work = update_given_work_estimates(work, avg_earn_per_diff_sat)
    if len(updated_work) > 0:
        # Save them
        for w in updated_work:
            db.work_update_nocommit(cursor, w)
    cursor.close()
    print(f"Updated estimates: {len(updated_work)}/{len(work)} workitem estimates updated")
    return len(updated_work)
*/

fn create_new_miner_record_if_needed(conn: &Connection, id: u32) -> Result<(), Box<dyn Error>> {
    if db::miner_ss_exists(conn, id)? {
        return Ok(());
    }
    let now_utc = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as u32;
    let user_s = db::userlookup_get_string(conn, id)?;
    let miner_ss = MinerSnapshot::new(id, user_s, now_utc, 0, 0, 0, 0, 0, -1, now_utc);
    let _ = db::miner_ss_insert_nocommit(conn, &miner_ss)?;
    Ok(())
}

fn create_new_miner_records_if_needed(
    conn: &mut Connection,
    affected_user_ids: &HashSet<u32>,
) -> Result<(), Box<dyn Error>> {
    let conntx = conn.transaction()?;
    for id in affected_user_ids {
        let _ = create_new_miner_record_if_needed(&conntx, *id)?;
    }
    let _ = conntx.commit()?;
    Ok(())
}

fn get_avg_block_earn(conn: &Connection) -> Result<f64, Box<dyn Error>> {
    let (sum_earn, sum_diff) = db::block_get_last_avg_n(conn, BLOCK_AVERAGE_EARNING_COUNT)?;
    let mut avg_earn = 0.0;
    if sum_diff > 0 {
        avg_earn = 1.0 / (sum_diff as f64) * (sum_earn as f64);
    }
    println!(
        "Avg earn from the {} last blocks (msats/128kdiff):  {:.1} ({} {})",
        BLOCK_AVERAGE_EARNING_COUNT,
        avg_earn * 1000.0 * 131072.0,
        sum_earn,
        sum_diff
    );
    Ok(avg_earn)
}

fn iteration(
    conn: &mut Connection,
    conn_workstat_ro: &Connection,
    conn_oceanmgr_ro: &Connection,
    status: &mut Status,
) -> Result<(), Box<dyn Error>> {
    let _ = get_status_status(conn, status)?;
    print_status(&status);

    let mut affected_user_ids = HashSet::<u32>::new();
    let cnt_wi = retrieve_new_workitems(conn_workstat_ro, conn, status, &mut affected_user_ids)?;

    let cnt_bl1 = count_new_blocks(conn_oceanmgr_ro, status)?;
    // println!("New block count: {cnt_bl1}");

    let cnt_new_payment = retrieve_new_payments(conn, status, &mut affected_user_ids)?;

    if cnt_wi == 0 && cnt_bl1 == 0 && cnt_new_payment == 0 {
        // println("No new data found");
        return Ok(());
    }

    let mut cnt_bl2 = 0;
    let mut new_blocks_accntd = 0;
    let mut tot_blocks_earned: u64 = 0;
    let mut tot_work_comm_pre: u64 = 0;
    let mut tot_work_comm_post: u64 = 0;
    let mut tot_work_estim_pre: u64 = 0;
    if cnt_wi > 0 || cnt_bl1 > 0 {
        let _cnt_bl1 = retrieve_new_blocks(conn, conn_oceanmgr_ro, status)?;
        print_status(&status);

        tot_blocks_earned = db::block_get_total_earned(conn)?;
        tot_work_comm_pre = db::work_get_total_committed(conn)?;

        (cnt_bl2, new_blocks_accntd) = process_new_blocks(conn, status, &mut affected_user_ids)?;
        print_status(&status);

        // Blocks processed (zero or more)
        tot_work_comm_post = db::work_get_total_committed(conn)?;
        tot_work_estim_pre = db::work_get_total_estimated(conn)?;

        // Check for consistency
        let expected_new_comm_msat = tot_blocks_earned * 1000;
        if expected_new_comm_msat != tot_work_comm_post {
            println!(
                "ERROR: Total work committed and blocks committed mismatch {} vs. {} diff {}   {} {} {}",
                tot_work_comm_post,
                expected_new_comm_msat,
                tot_work_comm_post as i64 - expected_new_comm_msat as i64,
                tot_work_comm_pre,
                tot_blocks_earned,
                new_blocks_accntd
            );
        }

        // if cnt_bl2 == 0 and cnt_new_payment == 0:
        //     print(f"iteration, but no new block and no new payment ({last_block_procd}, earned {tot_blocks_earned}) ({last_payment_procd})")
        //    return
    }

    // Some info changed, update snapshots
    let avg_earn = get_avg_block_earn(conn)?;

    let cnt_considered =
        update_work_estimates(conn, status.birth_time, avg_earn, &mut affected_user_ids)?;

    let tot_work_estim_post = db::work_get_total_estimated(conn)?;

    let _ = create_new_miner_records_if_needed(conn, &mut affected_user_ids)?;

    if cnt_bl2 > 0 {
        println!(
            "iteration: processed {} block(s) with {} msat, {}  {}  comm {} -> {} ({})  estim {} -> {} ({})",
            cnt_bl2,
            new_blocks_accntd,
            status.last_block_procd,
            tot_blocks_earned,
            tot_work_comm_pre,
            tot_work_comm_post,
            tot_work_comm_post - tot_work_comm_pre,
            tot_work_estim_pre,
            tot_work_estim_post,
            tot_work_estim_post - tot_work_estim_pre
        );
    }

    println!(
        "  new payments: {} ({})",
        cnt_new_payment, status.last_payment_procd
    );
    println!(
        "  users affected: {}  workitems considered: {}",
        affected_user_ids.len(),
        cnt_considered
    );

    // if len(affected_user_ids.keys()) >= 1:
    //     id = list(affected_user_ids.keys())[0]
    //     cursor = conn.cursor()
    //     print(f"    miner user id {id} {db.userlookup_get_string(cursor, id)} {db.work_get_user_total_committed(cursor, id)} {db.work_get_user_total_estimated(cursor, id)}")
    //     cursor.close()

    Ok(())
}

pub fn loop_iterations(
    conn: &mut Connection,
    conn_workstat_ro: &Connection,
    conn_oceanmgr_ro: &Connection,
) -> Result<(), Box<dyn Error>> {
    // global birth_time
    let birth_time = env::var("PAYCALC_BIRTH_TIME")
        .unwrap_or("0".to_string())
        .parse::<u32>()?;
    let mut status = Status::new(birth_time);

    println!("Paycalc/main: loop starting...");
    if birth_time > 0 {
        println!("Birth time: {birth_time}");
    }

    let sleep_secs: f64 = 5.0;
    let mut next_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();

    loop {
        match iteration(conn, conn_workstat_ro, conn_oceanmgr_ro, &mut status) {
            Ok(_) => {}
            Err(err) => {
                println!("ERROR in iteration, {}", err.to_string());
                continue;
            }
        }

        next_time = next_time + sleep_secs;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        let to_sleep = f64::max(next_time - now, 0.1);
        if to_sleep > 0.0 {
            // println!("Sleeping for {}  secs... (next_time {})", to_sleep, next_time);
            thread::sleep(Duration::from_secs_f64(to_sleep));
        }
    }
    // Ok(())
}
