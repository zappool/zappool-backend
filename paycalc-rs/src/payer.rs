use crate::cln_pay::pay_invoice;
use crate::common::{PaymentMethod, PaymentResult, shorten_id};
use crate::common_db::get_db_file;
use crate::db_pc as db;
use crate::dto_pc::{PayRequest, Payment};
use crate::error_codes::*;
use crate::ln_address::get_invoice_from_ln_address;
use crate::nostr_profile::get_nostr_ln_address;

use dotenv;
use rusqlite::Connection;

use std::error::Error;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const RETRY_DELAY: u32 = 600;
const PAYMENT_RETRIES_MAX: u32 = 10;

pub fn print_last_payments(conn: &Connection, period_days: u32) -> Result<(), Box<dyn Error>> {
    let now_utc = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as u32;
    let start_time = now_utc - period_days * 86400;
    let payms = db::payment_get_all_after_time(conn, start_time)?;
    println!("");
    println!("Recent payments ({} days): ({})", period_days, payms.len());
    for (pr, p) in &payms {
        let age_hr = ((now_utc - p.status_time) as f64) / 3600.0;
        println!(
            "  {}  {:.1} \t {} {} \t {} {} {} {} '{}' '{}'",
            shorten_id(&pr.pri_id),
            age_hr,
            pr.req_amnt,
            p.paid_amnt,
            p.status,
            p.retry_cnt,
            p.error_code,
            p.error_str,
            shorten_id(&p.secon_id),
            shorten_id(&p.terti_id)
        );
    }
    Ok(())
}

async fn pay_lightning_invoice(
    invoice: &str,
    req_amnt: u64,
    label: &str,
) -> Result<PaymentResult, Box<dyn Error>> {
    let res = pay_invoice(invoice, req_amnt, label).await?;
    Ok(res)
}

// Handle a lightning address payment
async fn process_lightning_address_payment(
    _paym: &Payment,
    pr: &PayRequest,
) -> Result<PaymentResult, Box<dyn Error>> {
    let ln_address = &pr.pri_id;
    let (success, invoice, err_nonfinal, err_str) =
        get_invoice_from_ln_address(&ln_address, pr.req_amnt).await?;
    if !success || invoice.is_none() {
        return if err_nonfinal {
            Ok(PaymentResult::new(
                false,
                true,
                ERROR_LN_ADDRESS_NONFINAL_FAILURE,
                &err_str,
                "",
                "",
                0,
                0,
                "",
            ))
        } else {
            Ok(PaymentResult::new(
                false,
                false,
                ERROR_LN_ADDRESS_FINAL_FAILURE,
                &err_str,
                "",
                "",
                0,
                0,
                "",
            ))
        };
    }
    // Success
    let invoice = invoice.unwrap();
    println!("Obtained LN invoice: ({invoice})");

    let mut pay_res = pay_lightning_invoice(&invoice, pr.req_amnt, &pr.pri_id).await?;

    pay_res.secon_id = invoice;
    pay_res.terti_id = "".to_string();
    pay_res.err_code = if !pay_res.success {
        if pay_res.err_nonfinal {
            ERROR_LN_BOLT11_INVOICE_NONFINAL_FAILURE
        } else {
            ERROR_LN_BOLT11_INVOICE_FINAL_FAILURE
        }
    } else {
        ERROR_OK
    };

    Ok(pay_res)
}

// Handle a Nostr lightning payment
async fn process_nostr_lightning_payment(
    _paym: &Payment,
    pr: &PayRequest,
) -> Result<PaymentResult, Box<dyn Error>> {
    let npub = &pr.pri_id;
    let ln_address = &get_nostr_ln_address(npub).await?.unwrap_or("".into());
    if ln_address.len() == 0 {
        let err_str = format!("Could not obtain LN Address for npub, '{ln_address}' '{npub}'");
        return Ok(PaymentResult::new(
            false,
            true,
            ERROR_NOSTR_LN_ADDRESS_NONFINAL_FAILURE,
            &err_str,
            "",
            "",
            0,
            0,
            "",
        ));
    }
    println!("Obtained LN Address: '{ln_address}'");

    let (success, invoice, err_nonfinal, err_str) =
        get_invoice_from_ln_address(ln_address, pr.req_amnt).await?;
    if !success || invoice.is_none() {
        if err_nonfinal {
            return Ok(PaymentResult::new(
                false,
                true,
                ERROR_LN_ADDRESS_NONFINAL_FAILURE,
                &err_str,
                ln_address,
                "",
                0,
                0,
                "",
            ));
        } else {
            return Ok(PaymentResult::new(
                false,
                false,
                ERROR_LN_ADDRESS_FINAL_FAILURE,
                &err_str,
                ln_address,
                "",
                0,
                0,
                "",
            ));
        }
    }
    let invoice = invoice.unwrap();
    // Success
    println!("Obtained LN invoice: ({invoice})");

    let mut pay_res = pay_lightning_invoice(&invoice, pr.req_amnt, &pr.pri_id).await?;
    pay_res.secon_id = ln_address.to_string();
    pay_res.terti_id = invoice;
    pay_res.err_code = if !pay_res.success {
        if pay_res.err_nonfinal {
            ERROR_LN_BOLT11_INVOICE_NONFINAL_FAILURE
        } else {
            ERROR_LN_BOLT11_INVOICE_FINAL_FAILURE
        }
    } else {
        ERROR_OK
    };

    Ok(pay_res)
}

//. Handle a payment by method
async fn process_payment_generic(
    paym: &Payment,
    pr: &PayRequest,
) -> Result<PaymentResult, Box<dyn Error>> {
    if pr.pay_method == PaymentMethod::PAYMENT_METHOD_LN_ADDRESS {
        return process_lightning_address_payment(paym, pr).await;
    }
    if pr.pay_method == PaymentMethod::PAYMENT_METHOD_NOSTR_LIGHTNING {
        return process_nostr_lightning_payment(paym, pr).await;
    }
    Ok(PaymentResult::new(
        false,
        false,
        ERROR_GENERIC_FINAL_FAILURE,
        &format!("Unknown payment method {}", pr.pay_method),
        "",
        "",
        0,
        0,
        "",
    ))
}

async fn process_payment_start(
    conn: &mut Connection,
    pr: &PayRequest,
    paym_orig: &Payment,
) -> Result<(), Box<dyn Error>> {
    let now_utc = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
        .floor() as u32;

    if paym_orig.id < 0 {
        let mut conntx = conn.transaction()?;
        let mut paym = Payment::new(
            -1,
            pr.id,
            now_utc,
            STATUS_NOTTRIED,
            now_utc,
            0,
            "".into(),
            0,
            0,
            "".into(),
            "".into(),
            0,
            0,
            0,
            "".into(),
        );
        paym.id = db::payment_update_or_insert_nocommit(&mut conntx, &paym)? as i32;
        let _ = conntx.commit()?;
    }

    if paym_orig.status == STATUS_FINAL_FAILURE || paym_orig.status == STATUS_SUCCESS_FINAL {
        println!(
            "WARNING: Payment is already final, ignoring ({})",
            paym_orig.status
        );
        return Ok(());
    }

    if paym_orig.status == STATUS_NONFINAL_FAILURE {
        let next_retry_time = paym_orig.fail_time + RETRY_DELAY;
        if now_utc < next_retry_time {
            // print(f"Payment was failed, retry cnt {paym.retry_cnt}, retrying later, in {next_retry_time - now_utc} secs")
            return Ok(());
        } else {
            println!(
                "Payment was failed, retry cnt {} {}, retrying now",
                paym_orig.retry_cnt, paym_orig.fail_time
            );
        }
    }

    println!(
        "Considering payment:  rid {} priid {} amnt {}   rid {} pid {} status {} errc {} errs {} retry {}",
        pr.id,
        pr.pri_id,
        pr.req_amnt,
        paym_orig.req_id,
        paym_orig.id,
        paym_orig.status,
        paym_orig.error_code,
        paym_orig.error_str,
        paym_orig.retry_cnt
    );

    if paym_orig.status == STATUS_IN_PROGRESS {
        println!("WARNING: Payment marked as in progress, ignoring...");
    }

    let mut paym = paym_orig.clone();
    paym.status = STATUS_IN_PROGRESS;
    paym.status_time = now_utc;
    let mut conntx = conn.transaction()?;
    let _ = db::payment_update_or_insert_nocommit(&mut conntx, &paym)?;
    let _ = conntx.commit()?;

    let pay_res = process_payment_generic(&paym, pr).await?;

    // Process and store error
    let now_utc = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
        .floor() as u32;
    let status;
    if !pay_res.success {
        // Error
        paym.retry_cnt = paym.retry_cnt + 1;
        paym.fail_time = now_utc;
        if pay_res.err_nonfinal {
            if paym.retry_cnt as u32 >= PAYMENT_RETRIES_MAX {
                println!("WARNING: Failing after {} retries!", paym.retry_cnt);
                status = STATUS_FINAL_FAILURE;
            } else {
                status = STATUS_NONFINAL_FAILURE;
            }
        } else {
            status = STATUS_FINAL_FAILURE;
        }
        paym.secon_id = pay_res.secon_id;
        paym.terti_id = pay_res.terti_id;
        paym.paid_amnt = 0;
        paym.paid_fee = 0;
        paym.pay_time = 0;
        paym.pay_ref = "".into();
    } else {
        status = STATUS_SUCCESS_FINAL;
        paym.secon_id = pay_res.secon_id;
        paym.terti_id = pay_res.terti_id;
        paym.paid_amnt = pay_res.paid_amount;
        paym.paid_fee = pay_res.paid_fee;
        paym.pay_time = now_utc;
        paym.pay_ref = pay_res.reference;
    }
    if paym.status != status {
        paym.status = status;
        paym.status_time = now_utc;
    }
    paym.error_code = pay_res.err_code;
    paym.error_str = pay_res.err_str;
    let mut conntx = conn.transaction()?;
    let _ = db::payment_update_or_insert_nocommit(&mut conntx, &paym)?;
    let _ = conntx.commit()?;
    if paym.status != STATUS_SUCCESS_FINAL {
        println!(
            "ERROR: There was an error in payment: {} {}  {} {} {} '{}'",
            paym.id,
            paym.req_id,
            paym.status,
            paym.error_code,
            pay_res.err_nonfinal,
            paym.error_str
        );
    } else {
        println!(
            "Successful payment: {} {}  {} {}",
            paym.id, paym.req_id, paym.paid_amnt, paym.pay_time
        );
    }

    Ok(())
}

async fn iteration(conn: &mut Connection) -> Result<(), Box<dyn Error>> {
    let open_requests = db::payreq_get_all_non_final(conn)?;
    if !open_requests.is_empty() {
        println!("Open pay requests: {}", open_requests.len());
        for (pr, paym) in &open_requests {
            let _ = process_payment_start(conn, pr, paym).await?;
        }
    }
    Ok(())
}

pub async fn loop_iterations() -> Result<(), Box<dyn Error>> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    let dbfile = get_db_file("paycalc.db", false);
    let mut conn = Connection::open(&dbfile)?;

    println!("Payer: loop starting");

    let sleep_secs = 5;
    let mut next_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();

    loop {
        match iteration(&mut conn).await {
            Ok(_) => {}
            Err(e) => {
                println!("ERROR in iteration, {:?}", e);
                continue;
            }
        };

        next_time = next_time + sleep_secs as f64;
        let now_utc = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        let to_sleep = f64::max(next_time - now_utc, 0.1);
        if to_sleep > 0.0 {
            // println!("Sleeping for {:.2} secs... (next_time {:.0})", to_sleep, next_time);
            thread::sleep(Duration::from_secs_f64(to_sleep));
        }
    }
    // Ok(())
}
