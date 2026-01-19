use crate::payment_method::{
    adjusted_primary_id, determine_payment_method, get_default_payment_method_from_env,
};

use common_rs::common_db::get_db_file;
use common_rs::db_pc as db;
use common_rs::dto_pc::{MinerSnapshot, PayRequest};
use payer::common::{PaymentMethod, shorten_id};

use dotenv;
use rusqlite::{Connection, Transaction};

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

//
// Create payrequests, at specified times (typ. daily)
//

/// The portion of earning considered for payout of the the only-estimated-not-committed amount
const PAYOUT_RATIO_FOR_ESTIMATED: f64 = 0.67;

fn print_miner_snapshot(ss: &MinerSnapshot) {
    println!(
        "  {}:{} \t {} \t {} \t {} \t {} \t {} \t {}",
        ss.user_id,
        shorten_id(&ss.user_s),
        ss.tot_commit,
        ss.tot_estimate,
        ss.tot_paid,
        ss.unpaid,
        ss.unpaid_cons,
        ss.payreq_id
    );
}

pub fn print_miner_snapshots(conn: &Connection) -> Result<(), Box<dyn Error>> {
    let snapshots = db::miner_ss_get_all(conn)?;
    println!("");
    println!("Snapshots: ({})", snapshots.len());
    println!("  miner \t committed \t estimated \t paid \t unpaid \t unpaid_cons \t payreqid");
    for ss in &snapshots {
        print_miner_snapshot(ss);
    }
    Ok(())
}

pub fn print_updated_miner_snapshots(conn: &Connection) -> Result<(), Box<dyn Error>> {
    let snapshots = db::miner_ss_get_all(conn)?;
    println!("");
    println!("Updated snapshots: ({})", snapshots.len());
    println!("  miner \t committed \t estimated \t paid \t unpaid \t unpaid_cons \t payreqid");
    for ss in &snapshots {
        let mut ss_copy = ss.clone();
        let changed = update_miner_snapshot(conn, &mut ss_copy)?;
        if changed {
            print!("*");
        } else {
            print!(" ");
        }
        let _ = print_miner_snapshot(&ss_copy);
    }
    Ok(())
}

pub fn print_pay_total_stats(conn: &Connection) -> Result<(), Box<dyn Error>> {
    println!();
    let (total_paid, total_paid_fee) = db::payment_get_total_amount(conn)?;
    let total_paid = ((total_paid as f64) / 1000.0).round() as u64;
    let total_paid_fee = ((total_paid_fee as f64) / 1000.0).round() as u32;
    println!("Total paid:  {total_paid} (plus {total_paid_fee} fees)");
    Ok(())
}

pub fn print_pay_requests(conn: &Connection) -> Result<(), Box<dyn Error>> {
    println!("");
    let open_pay_requests = db::payreq_get_all_non_final(conn)?;
    println!("Open pay requests: {}", open_pay_requests.len());
    for (pr, paym) in &open_pay_requests {
        print!("  {} {} {} {}", pr.id, pr.miner_id, pr.pri_id, pr.req_amnt,);
        if let Some(paym) = paym {
            println!(
                "  {} {} {} {} {} {}",
                paym.id,
                paym.req_id,
                paym.status,
                paym.retry_cnt,
                shorten_id(&paym.secon_id),
                shorten_id(&paym.terti_id)
            );
        } else {
            println!("  --");
        }
    }
    Ok(())
}

// Return PAYOUT_THRESHOLD_MSAT, PAYOUT_MAXIMUM_MSAT and PAYOUT_GRANULARITY_MSAT from env
fn get_payout_threshold() -> Result<(u64, u64, u32), Box<dyn Error>> {
    let mut threshold = env::var("PAYOUT_THRESHOLD_MSAT")
        .unwrap_or("5000".into())
        .parse::<u64>()?;
    let mut maximum = env::var("PAYOUT_MAXIMUM_MSAT")
        .unwrap_or("20000000".into())
        .parse::<u64>()?;
    let granularity = env::var("PAYOUT_GRANULARITY_MSAT")
        .unwrap_or("1000".into())
        .parse::<u32>()?;
    threshold = (((threshold as f64) / (granularity as f64)) * (granularity as f64)).ceil() as u64;
    maximum = (((maximum as f64) / (granularity as f64)) * (granularity as f64)).floor() as u64;
    // println!("{threshold} {maximum} {granularity}");
    Ok((threshold, maximum, granularity))
}

fn calculate_to_pay_for_miner(miner: &MinerSnapshot) -> Result<Option<u64>, Box<dyn Error>> {
    let (threshold, maximum, granularity) = get_payout_threshold()?;
    if miner.unpaid_cons < threshold as i64 {
        // too little, ignore for now
        return Ok(None);
    }
    assert!(miner.unpaid_cons > 0);
    let unpaid_cons = miner.unpaid_cons as u64; // it is non-negative by now
    // Clap to min, max
    let mut to_pay = std::cmp::min(std::cmp::max(unpaid_cons, threshold), maximum);
    // Round to granularity (typically sat)
    to_pay = granularity as u64 * ((to_pay as f64) / (granularity as f64)).round() as u64;
    Ok(Some(to_pay))
}

fn create_pay_request_if_needed(
    conn: &Transaction,
    miner: &mut MinerSnapshot,
    default_payment_method: PaymentMethod,
) -> Result<(), Box<dyn Error>> {
    let to_pay = calculate_to_pay_for_miner(miner)?;
    if to_pay.is_none() || to_pay.unwrap_or(0) == 0 {
        return Ok(());
    }
    let to_pay = to_pay.unwrap();

    let mut primary_id = miner.user_s.to_string();

    // Low-level substitution for test setup
    let substitute_from = env::var("DUMMY_SUBSTITUTE_LNADDR_FROM");
    let substitute_to = env::var("DUMMY_SUBSTITUTE_LNADDR_TO");
    if let Ok(substitute_from) = substitute_from {
        if let Ok(substitute_to) = substitute_to {
            if primary_id == substitute_from {
                primary_id = substitute_to.clone();
            }
        }
    }
    //println!(primary_id);

    let payment_method =
        determine_payment_method(miner.user_id, &primary_id, default_payment_method)?;
    let adj_primary_id = adjusted_primary_id(payment_method, &primary_id)?;
    if adj_primary_id != primary_id {
        println!("Adjusted primary id: {}  ({})", adj_primary_id, primary_id);
    }

    let pr = PayRequest::new(
        0,
        miner.user_id,
        to_pay,
        payment_method.to_string(),
        adj_primary_id,
        miner.time,
    );
    let pr_id = db::payreq_insert_nocommit(conn, &pr)?;
    miner.payreq_id = pr_id as i32;
    println!(
        "Payment request created, ID {}, user {}",
        miner.payreq_id, miner.user_s
    );
    Ok(())
}

// Compute updated committed/estimated/etc values for a miner snapshot
fn compute_unpaid_values(
    tot_committed: u64,
    tot_estimated: u64,
    tot_paid: u64,
) -> Result<(i64, i64), Box<dyn Error>> {
    let unpaid = tot_committed as i64 + tot_estimated as i64 - tot_paid as i64;
    let est_cons = ((tot_estimated as f64) * (PAYOUT_RATIO_FOR_ESTIMATED as f64)).floor() as i64;
    let unpaid_cons = tot_committed as i64 + est_cons - tot_paid as i64;
    Ok((unpaid, unpaid_cons))
}

// Compute updated committed/estimated/etc values for a miner snapshot
fn compute_miner_snapshot_values(
    conn: &Connection,
    user_id: u32,
) -> Result<(u64, u64, u64, i64, i64, u32), Box<dyn Error>> {
    let (tot_committed, tot_estimated, last_time) = db::work_get_user_totals(conn, user_id)?;
    let tot_paid = db::payment_get_total_paid_to_miner(conn, user_id)?;
    // println!("tot_paid {tot_paid} (id {id})");
    let (unpaid, unpaid_cons) = compute_unpaid_values(tot_committed, tot_estimated, tot_paid)?;
    Ok((
        tot_committed,
        tot_estimated,
        tot_paid,
        unpaid,
        unpaid_cons,
        last_time,
    ))
}

// Update miner snapshot values (totals)
fn update_miner_snapshot(
    conn: &Connection,
    ss: &mut MinerSnapshot,
) -> Result<bool, Box<dyn Error>> {
    let now_utc = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as u32;

    let (tot_committed, tot_estimated, tot_paid, unpaid, unpaid_cons, commit_last_time) =
        compute_miner_snapshot_values(conn, ss.user_id)?;
    if tot_committed == ss.tot_commit
        && tot_estimated == ss.tot_estimate
        && tot_paid == ss.tot_paid
        && unpaid == ss.unpaid
        && unpaid_cons == ss.unpaid_cons
        && commit_last_time == ss.commit_last_time
    {
        // No change
        return Ok(false);
    }

    // Update
    ss.tot_commit = tot_committed;
    ss.tot_estimate = tot_estimated;
    ss.tot_paid = tot_paid;
    ss.unpaid = unpaid;
    ss.unpaid_cons = unpaid_cons;
    ss.time = now_utc;
    ss.commit_last_time = commit_last_time;

    Ok(true)
}

/*
// Create updated miner snapshot, including totals.
fn generate_miner_snapshot(conn: &Connection, user_id: u32, user_s_opt: Option<String>, payreqid: i32) -> Result<(MinerSnapshot, bool), Box<dyn Error>> {
    let now_utc = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as u32;

    let (tot_committed, tot_estimated, tot_paid, unpaid, unpaid_cons) = compute_miner_snapshot_values(conn, user_id)?;

    let user_s = if let Some(u) = user_s_opt {
        u
    } else {
        db::userlookup_get_string(conn, user_id)?
    };

    let ms = MinerSnapshot::new(user_id, user_s, now_utc, tot_committed, tot_estimated, tot_paid, unpaid, unpaid_cons, payreqid);
    Ok(ms)
}
*/

// Update miner snapshots, including totals.
// Also computes amount scheduled for payment.
fn update_miner_snapshots_nocommit(conn: &Transaction) -> Result<u32, Box<dyn Error>> {
    let snapshots = db::miner_ss_get_all(conn)?;
    let mut cnt = 0;
    for ss in &snapshots {
        let mut ss_copy = ss.clone();
        let changed = update_miner_snapshot(conn, &mut ss_copy)?;
        if changed {
            let _ = db::miner_ss_insert_nocommit(conn, &ss_copy)?;
            cnt += 1;
            let _ = print_miner_snapshot(&ss_copy);
        }
    }
    Ok(cnt)
}

// Update miner snapshots, including totals.
// TODO: invoke only for changed items; but how?
// Also computes amount scheduled for payment.
fn update_miner_snapshots_and_create_payreqs(
    conn: &mut Connection,
    default_payment_method: PaymentMethod,
) -> Result<(), Box<dyn Error>> {
    let conntx = conn.transaction()?;
    let _ = update_miner_snapshots_nocommit(&conntx)?;

    // Record open pay requests, not to create new request for the same miners
    let open_pay_requests = db::payreq_get_all_non_final(&conntx)?;
    let mut miner_ids_with_open_pay_request = HashMap::<u32, PayRequest>::new();
    for (pr, _paym) in &open_pay_requests {
        let _ = miner_ids_with_open_pay_request.insert(pr.miner_id, pr.clone());
    }

    let mut snapshots = db::miner_ss_get_all(&conntx)?;
    let mut cnt = 0;
    for ss in &mut snapshots {
        let id = ss.user_id;

        if miner_ids_with_open_pay_request.contains_key(&id) {
            let pr = &miner_ids_with_open_pay_request[&id];
            println!(
                "WARNING: Miner {} already has a payrequest ({} {})",
                id, pr.id, pr.req_amnt
            );
        } else {
            let _ = create_pay_request_if_needed(&conntx, ss, default_payment_method)?;
        }
        cnt += 1;
    }

    let _ = conntx.commit()?;
    println!("Updated miner snapshots, cnt {cnt}");
    Ok(())
}

fn iteration(
    conn: &mut Connection,
    default_payment_method: PaymentMethod,
) -> Result<(), Box<dyn Error>> {
    let _ = update_miner_snapshots_and_create_payreqs(conn, default_payment_method)?;
    let _ = print_miner_snapshots(conn)?;
    let _ = print_pay_requests(conn)?;
    Ok(())
}

pub fn loop_iterations() -> Result<(), Box<dyn Error>> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    let dbfile = get_db_file("paycalc.db", false);
    let mut conn = Connection::open(&dbfile)?;

    let payout_period_secs = env::var("PAYOUT_PERIOD_SECS")
        .unwrap_or("86400".into())
        .parse::<u32>()?;
    let default_payment_method = get_default_payment_method_from_env()?;
    println!(
        "Paycalc/Payreq: loop starting, period {}, def pm {}",
        payout_period_secs,
        default_payment_method.to_string()
    );

    loop {
        let now_utc = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;
        let next_new_time = ((now_utc as f64) / (payout_period_secs as f64)).round() as u32
            * payout_period_secs
            + (payout_period_secs / 2);
        let diff = next_new_time - now_utc;
        println!(
            "Next payreq check time in {:.1} secs ({})",
            diff, next_new_time
        );
        loop {
            let now_utc = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as u32;
            if now_utc >= next_new_time {
                break;
            }
            let diff = next_new_time - now_utc;
            let to_wait = f64::max((0.90 * (diff as f64)).floor() + 0.05, 0.1);
            if to_wait >= 2.0 {
                println!(
                    "Sleeping for {:.1} secs... (next_time {:.1} {})",
                    to_wait, diff, next_new_time
                );
            }
            thread::sleep(Duration::from_secs_f64(to_wait));
        }

        // Time!
        match iteration(&mut conn, default_payment_method) {
            Ok(_) => break,
            Err(e) => println!("ERROR in iteration, {e}"),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_unpaid_values_basic_case() {
        // Basic test case with typical values
        let result = compute_unpaid_values(1000, 500, 200);
        assert!(result.is_ok());

        let (unpaid, unpaid_cons) = result.unwrap();
        assert_eq!(unpaid, 1300);
        assert_eq!(unpaid_cons, 1135);
    }

    #[test]
    fn test_compute_unpaid_values_zero_values() {
        // Test with all zero values
        let result = compute_unpaid_values(0, 0, 0);
        assert!(result.is_ok());

        let (unpaid, unpaid_cons) = result.unwrap();
        assert_eq!(unpaid, 0);
        assert_eq!(unpaid_cons, 0);
    }

    #[test]
    fn test_compute_unpaid_values_zero_committed() {
        let result = compute_unpaid_values(0, 500, 300);
        assert!(result.is_ok());

        let (unpaid, unpaid_cons) = result.unwrap();
        assert_eq!(unpaid, 200);
        assert_eq!(unpaid_cons, 35);
    }

    #[test]
    fn test_compute_unpaid_values_zero_committed_more_paid() {
        let result = compute_unpaid_values(0, 500, 2000);
        assert!(result.is_ok());

        let (unpaid, unpaid_cons) = result.unwrap();
        assert_eq!(unpaid, -1500);
        assert_eq!(unpaid_cons, -1665);
    }
}
