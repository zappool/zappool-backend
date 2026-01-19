use common_rs::common_db::get_db_file;

use rusqlite::{Connection, OpenFlags};
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

fn print_period(start_time: u64, end_time: u64) {
    println!(
        "Period: {} -- {}  ({})",
        start_time,
        end_time,
        end_time - start_time
    );
}

async fn count_zaps(
    conn: &Connection,
    start_time: u64,
    end_time: u64,
) -> Result<(), Box<dyn Error>> {
    println!("\nCount Zaps ...");

    print_period(start_time, end_time);

    let mut stmt = conn.prepare(
        "SELECT COUNT(PAYMENT.PaidAmnt) AS Count, SUM(PAYMENT.PaidAmnt) AS TotPaid \
            FROM PAYMENT \
            INNER JOIN PAYREQ ON PAYMENT.ReqId = PAYREQ.Id \
            WHERE PAYMENT.PayTime >= ?1 AND \
            PAYMENT.PayTime <= ?2 AND \
            PAYREQ.PayMethod == \"ZAP\" AND \
            PAYMENT.Status == 2 \
            ",
    )?;

    let res = stmt
        .query_map((start_time, end_time), |row| {
            let count = row.get::<_, u32>(0)?;
            let tot_paid = row.get::<_, u64>(1)?;
            Ok((count, tot_paid))
        })?
        .filter(|res| res.is_ok())
        .map(|res| res.unwrap())
        .collect::<Vec<(u32, u64)>>();
    // println!("len {}", res.len());
    if res.len() >= 1 {
        let count = res[0].0;
        let tot_paid = res[0].1;
        println!("  Zap count: {}", count);
        println!(
            "  Zap total amount: {} sat  {} msat",
            tot_paid / 1000,
            tot_paid
        );
    }
    Ok(())
}

async fn count_users_with_zaps(
    conn: &Connection,
    start_time: u64,
    end_time: u64,
    min_zaps_threshold: u32,
) -> Result<(), Box<dyn Error>> {
    println!("\nCount users with zaps ...");

    print_period(start_time, end_time);

    let mut stmt = conn.prepare(
        "SELECT USERLOOKUP.String AS Miner, COUNT(USERLOOKUP.Id) AS Count, SUM(PAYMENT.PaidAmnt) AS TotPaid \
            FROM PAYMENT \
            INNER JOIN PAYREQ ON PAYMENT.ReqId = PAYREQ.Id \
            INNER JOIN USERLOOKUP ON PAYREQ.MinerId == USERLOOKUP.Id \
            WHERE PAYMENT.PayTime >= ?1 AND \
            PAYMENT.PayTime <= ?2 AND \
            PAYREQ.PayMethod == \"ZAP\" AND \
            PAYMENT.Status == 2 \
            GROUP BY USERLOOKUP.Id \
            ",
    )?;

    let res = stmt
        .query_map((start_time, end_time), |row| {
            let miner = row.get::<_, String>(0)?;
            let count = row.get::<_, u32>(1)?;
            let tot_paid = row.get::<_, u64>(2)?;
            Ok((miner, count, tot_paid))
        })?
        .filter(|res| res.is_ok())
        .map(|res| res.unwrap())
        .collect::<Vec<(String, u32, u64)>>();
    // println!("len {}", res.len());
    let mut count = 0;
    for r in res {
        let user = r.0;
        let zap_count = r.1;
        let tot_zap = r.2;
        if zap_count >= min_zaps_threshold {
            count += 1;
            println!(
                "  {} {} {}",
                zap_count,
                (tot_zap as f64 / 1000.0).round(),
                user
            );
        }
    }
    println!("{} entries", count);

    Ok(())
}

async fn list_stale_users_unpaid_amounts(
    conn: &Connection,
    now: u64,
) -> Result<(), Box<dyn Error>> {
    println!("\nList stale users and unpaid amounts ...");

    let min_unpaid = 5000;
    // TODO Use CommitLastTime instead of Time; more accurate
    let mut stmt = conn.prepare(
        "SELECT MINER_SS.UserS as User, MINER_SS.CommitLastTime as LastTime, MINER_SS.UnpaidCons as Unpaid \
            FROM MINER_SS \
            WHERE MINER_SS.UnpaidCons >= ?1 \
            ",
    )?;

    let res = stmt
        .query_map((min_unpaid,), |row| {
            let user = row.get::<_, String>(0)?;
            let time = row.get::<_, u64>(1)?;
            let unpaid = row.get::<_, u64>(2)?;
            Ok((user, time, unpaid))
        })?
        .filter(|res| res.is_ok())
        .map(|res| res.unwrap())
        .collect::<Vec<(String, u64, u64)>>();
    // println!("len {}", res.len());
    println!("  age (hr)  unpaid (sat)  user");
    for r in &res {
        let user = &r.0;
        let last_time = r.1;
        let age = (now - last_time) as f64 / 3600.0;
        let unpaid = r.2 as f64 / 1000.0;
        println!("  {:.2} h  {:.1}  {}", age, unpaid, user);
    }
    println!("{} entries", res.len());

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // paycalc_rs::ln_address::do_try().await;

    let dbfile = get_db_file("paycalc.db", false);
    let conn = Connection::open_with_flags(dbfile, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

    let now_utc = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let _ = count_zaps(&conn, now_utc - 10000 * 86400, now_utc).await?;

    let days = 5;
    let _ = count_users_with_zaps(&conn, now_utc - days * 86400 - 2 * 3600, now_utc, 3).await?;

    let _ = list_stale_users_unpaid_amounts(&conn, now_utc).await?;

    Ok(())
}
