use crate::common::PaymentResult;
use crate::ln_address::ln_p_url_from_address;
use crate::nostr_profile::get_nostr_ln_address;
use crate::payer::pay_lightning_invoice;

use common_rs::error_codes::*;

use bech32::{FromBase32, ToBase32, encode};
use nostr::nips::nip57::ZapRequestData;
use nostr::secp256k1::Secp256k1;
use nostr::util::JsonUtil;
use nostr::{EventBuilder, Keys, PublicKey, RelayUrl, SecretKey};

use std::error::Error;
use std::str::FromStr;

#[derive(Debug, serde::Deserialize)]
struct LnurlResponseData {
    callback: Option<String>,
    #[serde(rename = "allowsNostr")]
    allow_nostr: Option<bool>,
    #[serde(rename = "nostrPubkey")]
    nostr_pubkey: Option<String>,
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
// In case of error, return:
// - if the error is nonfinal
// - the error
pub async fn get_zap_invoice(
    ln_address: &str,
    amount_msats: u64,
    zap_event_str: &str,
) -> Result<String, (bool, Box<dyn Error>)> {
    // Retrieve a BOLT11 invoice from a Lightning Address.
    //
    // Args:
    //     ln_address: A Lightning Address in the format "user@domain"
    //     amount_msats: Amount to pay in millisatoshis
    //
    // Returns:
    //     A BOLT11 invoice string or raises an exception if the process fails

    println!("Processing LN address {ln_address} ...");

    // Step 1: Construct and request the Lightning Address URL
    let lnurlp_url = &ln_p_url_from_address(ln_address).map_err(|e| (false, e.into()))?;

    // Make the initial request to get the LNURL metadata
    let resp = match reqwest::get(lnurlp_url).await {
        Err(e) => {
            return Err((
                true,
                format!("HTTP request failed: {} {:?}", lnurlp_url, e).into(),
            ));
        }
        Ok(r) => r,
    };
    if resp.status() != reqwest::StatusCode::OK {
        return Err((
            true,
            format!("HTTP request failed: {} {}", resp.status(), lnurlp_url).into(),
        ));
    }
    let lnurlp_data = resp
        .json::<LnurlResponseData>()
        .await
        .map_err(|e| (true, e.into()))?;
    // println!("lnurlp_data {:?}", lnurlp_data);

    // Extract the callback URL
    let callback_url = match lnurlp_data.callback {
        None => {
            return Err((
                true,
                format!("Missing callback: {} {:?}", lnurlp_url, lnurlp_data).into(),
            ));
        }
        Some(c) => c,
    };
    // println!("callback {}", callback_url);

    println!(
        "allowsNostr {}  nostr pubkey '{}'",
        lnurlp_data.allow_nostr.unwrap_or(false),
        lnurlp_data.nostr_pubkey.unwrap_or_default()
    );

    // Check if the callback URL supports the specified amount
    let min_sendable = lnurlp_data.min_sendable.unwrap_or(1);
    let max_sendable = lnurlp_data.max_sendable.unwrap_or(u64::max_value());

    if amount_msats < min_sendable {
        return Err((
            false,
            format!("Amount {amount_msats} is below the minimum allowed: {min_sendable}").into(),
        ));
    }
    if amount_msats > max_sendable {
        return Err((
            false,
            format!("Amount {amount_msats} is above the maximum allowed: {max_sendable}").into(),
        ));
    }

    // let lnurl_bech32 = lnurlp_url.
    // str(lnurl.Lnurl(lnaddr_url).bech32).lower()
    // print(lnurl_bech32)

    let zap_event_serialized_quoted = urlencoding::encode(&zap_event_str);
    println!("Zap event as string: '{zap_event_serialized_quoted}' '{zap_event_str}'");

    let lnurlp_url_bech = encode(
        "lnurl",
        lnurlp_url.as_bytes().to_base32(),
        bech32::Variant::Bech32,
    )
    .map_err(|e| (false, e.into()))?;

    // TODO

    // Step 2: Make the callback request with the amount
    // Some providers expect amount in msats, others in sats - we'll use msats as that's our input
    let callback_with_amount = &format!(
        "{callback_url}?amount={amount_msats}&lnurl={lnurlp_url_bech}&nostr={zap_event_serialized_quoted}"
    );
    // println!("callback_with_amount {callback_with_amount}");

    let resp = match reqwest::get(callback_with_amount).await {
        Err(e) => {
            return Err((
                true,
                format!("HTTP request failed: {} {:?}", callback_with_amount, e).into(),
            ));
        }
        Ok(r) => r,
    };

    let callback_data = resp
        .json::<CallbackResponseData>()
        .await
        .map_err(|e| (true, e.into()))?;
    // println!("callback_data {:?}", callback_data);

    // Check if the response contains a BOLT11 invoice
    let invoice = match callback_data.pr {
        None => {
            return Err((
                false,
                format!("Invalid callback response: missing 'pr' field (BOLT11 invoice)").into(),
            ));
        }
        Some(i) => i,
    };

    // Return the BOLT11 invoice
    Ok(invoice)
}

/// Helper: npub from nsec
fn npub_from_secret_obj(secret_key: &SecretKey) -> Result<String, Box<dyn Error>> {
    let secp = Secp256k1::new();
    let pubkey = secret_key.x_only_public_key(&secp).0.serialize();
    let npub = encode("npub", pubkey.to_base32(), bech32::Variant::Bech32)?;
    Ok(npub)
}

/// Helper: npub from nsec vec
pub fn npub_from_secret_vec(secret_key_vec: &Vec<u8>) -> Result<String, Box<dyn Error>> {
    let secret_key_obj = SecretKey::from_slice(&secret_key_vec)?;
    npub_from_secret_obj(&secret_key_obj)
}

pub async fn nostr_zap(
    amount_msat: u64,
    sender_nsec_vec: &Vec<u8>,
    rec_npub: &str,
    relays: &Vec<&str>,
) -> Result<PaymentResult, (bool, Box<dyn Error>)> {
    let sender_nsec = SecretKey::from_slice(sender_nsec_vec).map_err(|e| (false, e.into()))?;
    let sender_npub = npub_from_secret_obj(&sender_nsec).map_err(|e| (false, e.into()))?;
    println!("nostr_zap:  {amount_msat}  from {sender_npub}  to {rec_npub}");

    let ln_address = get_nostr_ln_address(rec_npub)
        .await
        .map_err(|e| (true, e.into()))?;
    if ln_address.len() == 0 {
        return Err((
            false,
            format!("Could not obtain LN Address for npub '{rec_npub}'").into(),
        ));
    }
    println!("Obtained LN Address: '{ln_address}'");

    let lnurlp_url_str = ln_p_url_from_address(&ln_address).map_err(|e| (false, e.into()))?;
    let lnurlp_url_bech = encode(
        "lnurl",
        lnurlp_url_str.as_bytes().to_base32(),
        bech32::Variant::Bech32,
    )
    .map_err(|e| (false, e.into()))?;

    let rec_npub_parse = bech32::decode(rec_npub).map_err(|e| (false, e.into()))?;
    let rec_npub_bytes =
        Vec::<u8>::from_base32(&rec_npub_parse.1).map_err(|e| (false, e.into()))?;
    let rec_pubkey = PublicKey::from_slice(&rec_npub_bytes).map_err(|e| (false, e.into()))?;
    let mut relay_urls = Vec::new();
    for rs in relays {
        let relay = RelayUrl::from_str(rs).map_err(|e| (false, e.into()))?;
        relay_urls.push(relay);
    }
    let mut zap_req_data = ZapRequestData::new(rec_pubkey, relay_urls);
    zap_req_data.amount = Some(amount_msat);
    zap_req_data.lnurl = Some(lnurlp_url_bech);

    println!("zap_req_data: {:?}", zap_req_data);
    let builder = EventBuilder::public_zap_request(zap_req_data);
    let zap_event = builder
        .sign_with_keys(&Keys::new(sender_nsec.clone()))
        .map_err(|e| (false, e.into()))?;
    let zap_event_serialized = &zap_event.as_json().to_string();

    let invoice = get_zap_invoice(&ln_address, amount_msat, &zap_event_serialized).await?;
    println!("Obtained ZAP invoice to be paid:   '{invoice}'");

    let mut pay_res = pay_lightning_invoice(&invoice, amount_msat, &rec_npub)
        .await
        .map_err(|e| (true, e.into()))?;

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_npub_from_secret_vec() {
        let dummy_nsec_vec = [7u8; 32].to_vec();
        let npub = npub_from_secret_vec(&dummy_nsec_vec).unwrap();
        assert_eq!(
            npub,
            "npub1nzwqkakt2cuhrlwfhme3asrvx4s0xfyadm57tkpu2a39t9hqtahs7fsn89"
        );
    }
}
