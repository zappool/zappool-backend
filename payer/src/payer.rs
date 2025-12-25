use crate::cln_pay::pay_invoice;
use crate::common::{PayerParameters, PaymentMethod, PaymentResult, shorten_id};
use crate::ln_address::get_invoice_from_ln_address;
use crate::nostr_profile::get_nostr_ln_address;
use crate::nostr_zap::{nostr_zap, npub_from_secret_vec};

use common_rs::common_db::get_db_file;
use common_rs::db_pc as db;
use common_rs::dto_pc::{PayRequest, Payment};
use common_rs::error_codes::*;

use dotenv;
use nostr::key::SecretKey;
use rusqlite::Connection;
use seedstore::KeyStore;

use std::env;
use std::error::Error;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const RETRY_DELAY: u32 = 600;
const PAYMENT_RETRIES_MAX: u32 = 10;
const DEFAULT_SECRET_FILE: &str = "secret.nsec";

pub fn get_nostr_secret_from_config() -> Result<Vec<u8>, Box<dyn Error>> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    let nsec_password = env::var("NOSTR_NSEC_FILE_PASSWORD").unwrap_or("MISSING".to_owned());

    let keystore = KeyStore::new_from_encrypted_file(DEFAULT_SECRET_FILE, &nsec_password)?;
    let nsec1 = keystore
        .get_secret_private_key()
        ?.secret_bytes().to_vec();
    Ok(nsec1)
}

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
            "  {}  {:.1} \t {} {} \t {:4} {} {} {} {} '{}' '{}'",
            shorten_id(&pr.pri_id),
            age_hr,
            pr.req_amnt,
            p.paid_amnt,
            pr.pay_method,
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

pub async fn pay_lightning_invoice(
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
    match get_invoice_from_ln_address(&ln_address, pr.req_amnt).await {
        Err((err_nonfinal, err)) => {
            if err_nonfinal {
                Ok(PaymentResult::new(
                    false,
                    true,
                    ERROR_LN_ADDRESS_NONFINAL_FAILURE,
                    &err.to_string(),
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
                    &err.to_string(),
                    "",
                    "",
                    0,
                    0,
                    "",
                ))
            }
        }
        Ok(invoice) => {
            // Success
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
    }
}

// Handle a Nostr lightning payment
async fn process_nostr_lightning_payment(
    _paym: &Payment,
    pr: &PayRequest,
) -> Result<PaymentResult, Box<dyn Error>> {
    let npub = &pr.pri_id;
    let ln_address = match get_nostr_ln_address(npub).await {
        Err(e) => {
            return Ok(PaymentResult::new(
                false,
                true,
                ERROR_NOSTR_LN_ADDRESS_NONFINAL_FAILURE,
                &e.to_string(),
                "",
                "",
                0,
                0,
                "",
            ));
        }
        Ok(a) => a,
    };
    println!("Obtained LN Address: '{ln_address}'");

    match get_invoice_from_ln_address(&ln_address, pr.req_amnt).await {
        Err((err_nonfinal, err)) => {
            if err_nonfinal {
                return Ok(PaymentResult::new(
                    false,
                    true,
                    ERROR_LN_ADDRESS_NONFINAL_FAILURE,
                    &err.to_string(),
                    &ln_address,
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
                    &err.to_string(),
                    &ln_address,
                    "",
                    0,
                    0,
                    "",
                ));
            }
        }
        Ok(invoice) => {
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
    }
}

// Handle a Nostr Zap payment
async fn process_nostr_zap_payment(
    _paym: &Payment,
    pr: &PayRequest,
    payer_params: &PayerParameters,
) -> Result<PaymentResult, Box<dyn Error>> {
    let sender_nsec = SecretKey::from_slice(&payer_params.nostr_secret_key)?;
    let rec_npub = &pr.pri_id;
    // TODO dynamic list
    let relays = vec![
        "wss://relay.primal.net/",
        "wss://relay.damus.io/",
        "wss://nos.lol/",
    ];

    match nostr_zap(pr.req_amnt, &sender_nsec, rec_npub, &relays).await {
        Err((non_final, err)) => {
            return if non_final {
                Ok(PaymentResult::new(
                    false,
                    true,
                    ERROR_NOSTR_ZAP_NONFINAL_FAILURE,
                    &err.to_string(),
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
                    ERROR_NOSTR_ZAP_FINAL_FAILURE,
                    &err.to_string(),
                    "",
                    "",
                    0,
                    0,
                    "",
                ))
            };
        }
        Ok(res) => Ok(res),
    }
}

//. Handle a payment by method
async fn process_payment_generic(
    paym: &Payment,
    pr: &PayRequest,
    payer_params: &PayerParameters,
) -> Result<PaymentResult, Box<dyn Error>> {
    if pr.pay_method == PaymentMethod::PmLnAddress.to_string() {
        return process_lightning_address_payment(paym, pr).await;
    }
    if pr.pay_method == PaymentMethod::PmNostrLightning.to_string() {
        return process_nostr_lightning_payment(paym, pr).await;
    }
    if pr.pay_method == PaymentMethod::PmNostrZap.to_string() {
        return process_nostr_zap_payment(paym, pr, payer_params).await;
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

fn save_payment(conn: &mut Connection, paym: &mut Payment) -> Result<(), Box<dyn Error>> {
    let mut conntx = conn.transaction()?;
    let id = db::payment_update_or_insert_nocommit(&mut conntx, &paym)? as i32;
    if paym.id != id {
        paym.id = id;
        // println!("save_payment: updated id: {}", paym.id);
    }
    let _ = conntx.commit()?;
    Ok(())
}

async fn process_payment_start(
    payer_params: &PayerParameters,
    conn: &mut Connection,
    pr: &PayRequest,
    paym_orig: &Option<Payment>,
) -> Result<(), Box<dyn Error>> {
    let now_utc = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
        .floor() as u32;

    let mut paym = match paym_orig {
        Some(p) => p.clone(),
        None => {
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
            let _ = save_payment(conn, &mut paym)?;
            paym
        }
    };

    if paym.status == STATUS_FINAL_FAILURE || paym.status == STATUS_SUCCESS_FINAL {
        println!(
            "WARNING: Payment is already final, ignoring ({})",
            paym.status
        );
        return Ok(());
    }

    if paym.status == STATUS_NONFINAL_FAILURE {
        let next_retry_time = paym.fail_time + RETRY_DELAY;
        if now_utc < next_retry_time {
            // print(f"Payment was failed, retry cnt {paym.retry_cnt}, retrying later, in {next_retry_time - now_utc} secs")
            return Ok(());
        } else {
            println!(
                "Payment was failed, retry cnt {} {}, retrying now",
                paym.retry_cnt, paym.fail_time
            );
        }
    }

    println!(
        "Considering payment:  rid {} priid {} amnt {}   rid {} pid {} status {} errc {} errs {} retry {}",
        pr.id,
        pr.pri_id,
        pr.req_amnt,
        paym.req_id,
        paym.id,
        paym.status,
        paym.error_code,
        paym.error_str,
        paym.retry_cnt
    );

    if paym.status == STATUS_IN_PROGRESS {
        println!("WARNING: Payment marked as in progress, ignoring...");
    }

    paym.status = STATUS_IN_PROGRESS;
    paym.status_time = now_utc;

    let _ = save_payment(conn, &mut paym)?;

    let pay_res = process_payment_generic(&paym, pr, payer_params).await?;

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

    let _ = save_payment(conn, &mut paym)?;

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

async fn iteration(payer_params: &PayerParameters, conn: &mut Connection) -> Result<(), Box<dyn Error>> {
    let open_requests = db::payreq_get_all_non_final(conn)?;
    if !open_requests.is_empty() {
        println!("Open pay requests: {}", open_requests.len());
        for (pr, paym) in &open_requests {
            let _ = process_payment_start(payer_params, conn, pr, paym).await?;
        }
    }
    Ok(())
}

pub async fn loop_iterations() -> Result<(), Box<dyn Error>> {
    println!("Payer: initializing ...");

    let nostr_secret_key = get_nostr_secret_from_config()?;
    let nostr_pub = npub_from_secret_vec(&nostr_secret_key)?;
    println!("Nostr secret key read from config, npub: {nostr_pub}");

    let payer_params = PayerParameters {
        nostr_secret_key,
    };

    // Load environment variables from .env file
    dotenv::dotenv().ok();

    let dbfile = get_db_file("paycalc.db", false);
    let mut conn = Connection::open(&dbfile)?;

    println!("Payer: loop starting ...");

    let sleep_secs = 5;
    let mut next_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();

    loop {
        match iteration(&payer_params, &mut conn).await {
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
