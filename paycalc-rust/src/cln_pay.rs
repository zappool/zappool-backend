use crate::common::PaymentResult;

use cln_rpc::ClnRpc;
use cln_rpc::model::{requests, responses};
use hex_conservative::display::DisplayHex;

use std::env;
use std::error::Error;
use std::fs;

// from pyln.client import LightningRpc
// import os

fn get_rpc_path() -> Result<String, Box<dyn Error>> {
    // Create an instance of the LightningRpc object
    // clnrpc = LightningRpc("/tmp/lightning1/lightning-rpc")
    let user = env::var("USER")?;
    // print(f"user: {user}")
    let rpc_pipe_path = format!("/home/{user}/.lightning/{user}/lightning-rpc");
    // print(f"rpc_pipe_path {rpc_pipe_path}")
    if !fs::exists(&rpc_pipe_path).unwrap_or(false) {
        let msg = format!(
            "ERROR: RPC pipe doesn't exists, CLN not runnig or not accessible, pipe '{rpc_pipe_path}' user '{user}'"
        );
        println!("{msg}");
        Err(msg.into())
    } else {
        Ok(rpc_pipe_path)
    }
}

/// May throw
pub async fn pay_invoice(
    invoice: &str,
    _amnt_msat: u64,
    label: &str,
) -> Result<PaymentResult, Box<dyn Error>> {
    let rpc_pipe_path = match get_rpc_path() {
        Err(e) => {
            return Ok(PaymentResult::new(
                false,
                true,
                0,
                &e.to_string(),
                "",
                "",
                0,
                0,
                "",
            ));
        }
        Ok(p) => p,
    };
    let mut rpc = ClnRpc::new(rpc_pipe_path).await?;

    let info_req = requests::GetinfoRequest {};
    let info_resp: responses::GetinfoResponse = rpc.call_typed(&info_req).await?;
    println!("info {:?}", info_resp);
    // funds = clnrpc.listfunds()
    // print(funds)

    let pay_req = requests::PayRequest {
        bolt11: invoice.to_string(),
        label: Some(label.to_string()),
        amount_msat: None,
        description: None,
        exclude: None,
        exemptfee: None,
        localinvreqid: None,
        maxdelay: None,
        maxfee: None,
        maxfeepercent: None,
        partial_msat: None,
        retry_for: None,
        riskfactor: None,
    };
    let pay_resp: responses::PayResponse = rpc.call_typed(&pay_req).await?;
    println!("info {:?}", info_resp);

    let status = pay_resp.status;
    let amount_sent_msat = pay_resp.amount_sent_msat.msat();
    let amount_msat = pay_resp.amount_msat.msat();
    let payment_hash = pay_resp.payment_hash.to_string();
    let payment_preimage = pay_resp.payment_preimage.to_vec().to_lower_hex_string();
    println!(
        "DEBUG: CLN response: {:?} {} {} {} {} {:?}",
        status, amount_sent_msat, amount_msat, payment_hash, payment_preimage, pay_resp
    );

    if status != responses::PayStatus::COMPLETE {
        let errstr = format!("ERROR: Non-complete status, {:?}", status);
        println!("{errstr}");
        return Ok(PaymentResult::new(
            false, true, 0, &errstr, "", "", 0, 0, "",
        ));
    }

    let fee = (amount_sent_msat - amount_msat) as u32;
    let reference = format!("{payment_preimage} {payment_hash}");

    Ok(PaymentResult::new(
        true,
        false,
        0,
        "OK",
        "",
        "",
        amount_sent_msat,
        fee,
        &reference,
    ))
}

// Get node info
#[allow(dead_code)]
async fn get_info() -> Result<responses::GetinfoResponse, Box<dyn Error>> {
    let rpc_pipe_path = match get_rpc_path() {
        Err(e) => {
            return Err(format!("CLN not runnig or not accessible, {:?}", e).into())
        }
        Ok(p) => p,
    };
    let mut rpc = ClnRpc::new(rpc_pipe_path).await?;

    let info_req = requests::GetinfoRequest {};
    let info_resp: responses::GetinfoResponse = rpc.call_typed(&info_req).await?;
    println!("info {:?}", info_resp);

    Ok(info_resp)
}

// Get funds info
async fn get_funds_info() -> Result<responses::ListfundsResponse, Box<dyn Error>> {
    let rpc_pipe_path = match get_rpc_path() {
        Err(e) => {
            return Err(format!("CLN not runnig or not accessible, {:?}", e).into())
        }
        Ok(p) => p,
    };
    let mut rpc = ClnRpc::new(rpc_pipe_path).await?;

    let funds_req = requests::ListfundsRequest { spent: None };
    let funds_resp: responses::ListfundsResponse = rpc.call_typed(&funds_req).await?;
    println!("info {:?}", funds_resp);
    Ok(funds_resp)
}

pub async fn print_node_info() -> Result<(), Box<dyn Error>> {
    // info = get_info()
    // print("LN node info:")
    // print(info)
    let funds = get_funds_info().await?;
    println!("");
    println!("LN node funds info:");
    println!("{:?}", funds);
    Ok(())
}
