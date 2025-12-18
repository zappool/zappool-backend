use std::error::Error;

#[derive(Debug, serde::Deserialize)]
struct LnurlResponseData {
    callback: Option<String>,
    #[serde(rename = "minSendable")]
    min_sendable: Option<u64>,
    #[serde(rename = "maxSendable")]
    max_sendable: Option<u64>,
}

#[derive(Debug, serde::Deserialize)]
struct CallbackResponseData {
    pr: Option<String>,
}

// Retrieve a BOLT11 incvoice from a Lightning Address
// Return:
// - success flag
// - invoice
// - if the error is nonfinal
// - error text
pub async fn get_invoice_from_ln_address(
    ln_address: &str,
    amount_msats: u64,
) -> Result<(bool, Option<String>, bool, String), Box<dyn Error>> {
    // Retrieve a BOLT11 invoice from a Lightning Address.
    //
    // Args:
    //     ln_address: A Lightning Address in the format "user@domain"
    //     amount_msats: Amount to pay in millisatoshis
    //
    // Returns:
    //     A BOLT11 invoice string or raises an exception if the process fails

    println!("Processing LN address {ln_address} ...");

    // Parse the Lightning Address
    let parts = ln_address.split("@").collect::<Vec<&str>>();
    if parts.len() < 2 {
        return Ok((
            false,
            None,
            false,
            format!("Invalid Lightning Address format: '{ln_address}'"),
        ));
    }
    let username = parts[0];
    let domain = parts[1];

    // Step 1: Construct and request the Lightning Address URL
    let lnurlp_url = &format!("https://{domain}/.well-known/lnurlp/{username}");

    // Make the initial request to get the LNURL metadata
    let resp = match reqwest::get(lnurlp_url).await {
        Err(e) => {
            return Ok((
                false,
                None,
                true,
                format!("HTTP request failed: {} {:?}", lnurlp_url, e),
            ));
        }
        Ok(r) => r,
    };
    if resp.status() != reqwest::StatusCode::OK {
        return Ok((
            false,
            None,
            true,
            format!("HTTP request failed: {} {}", resp.status(), lnurlp_url),
        ));
    }
    let lnurlp_data = resp.json::<LnurlResponseData>().await?;
    // println!("lnurlp_data {:?}", lnurlp_data);

    // Extract the callback URL
    let callback_url = match lnurlp_data.callback {
        None => {
            return Ok((
                false,
                None,
                true,
                format!("Missing callback: {} {:?}", lnurlp_url, lnurlp_data),
            ));
        }
        Some(c) => c,
    };
    // println!("callback {}", callback_url);

    // Check if the callback URL supports the specified amount
    let min_sendable = lnurlp_data.min_sendable.unwrap_or(1);
    let max_sendable = lnurlp_data.max_sendable.unwrap_or(u64::max_value());

    if amount_msats < min_sendable {
        return Ok((
            false,
            None,
            false,
            format!("Amount {amount_msats} is below the minimum allowed: {min_sendable}"),
        ));
    }
    if amount_msats > max_sendable {
        return Ok((
            false,
            None,
            false,
            format!("Amount {amount_msats} is above the maximum allowed: {max_sendable}"),
        ));
    }

    // Step 2: Make the callback request with the amount
    // Some providers expect amount in msats, others in sats - we'll use msats as that's our input
    let callback_with_amount = &format!("{callback_url}?amount={amount_msats}");
    // println!("callback_with_amount {callback_with_amount}");

    let resp = match reqwest::get(callback_with_amount).await {
        Err(e) => {
            return Ok((
                false,
                None,
                true,
                format!("HTTP request failed: {} {:?}", callback_with_amount, e),
            ));
        }
        Ok(r) => r,
    };

    let callback_data = resp.json::<CallbackResponseData>().await?;
    // println!("callback_data {:?}", callback_data);

    // Check if the response contains a BOLT11 invoice
    let invoice = match callback_data.pr {
        None => {
            return Ok((
                false,
                None,
                false,
                format!("Invalid callback response: missing 'pr' field (BOLT11 invoice)"),
            ));
        }
        Some(i) => i,
    };

    // Return the BOLT11 invoice
    Ok((true, Some(invoice), false, "".to_string()))
}

pub async fn do_try() {
    let (_success, invoice, _nonfinal, _error) = get_invoice_from_ln_address(
        "npub12rv5lskctqxxs2c8rf2zlzc7xx3qpvzs3w4etgemauy9thegr43sf485vg@npub.cash",
        5000,
    )
    .await
    .unwrap();
    println!("Invoice: {}", invoice.unwrap());
}
