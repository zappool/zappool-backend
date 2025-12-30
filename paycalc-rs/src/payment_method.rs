/// Payment method
use payer::common::{PaymentMethod, payment_methods};

use std::env;
use std::error::Error;
use std::str::FromStr;

fn get_user_method_setting_override_from_envstr(
    userid: u32,
    envstr: &str,
) -> Option<PaymentMethod> {
    let users = envstr.split(",");
    for u in users.into_iter() {
        let parts: Vec<_> = u.split(":").collect();
        if parts.len() == 2 {
            if userid.to_string() == parts[0] {
                return PaymentMethod::from_str(parts[1]).ok();
            }
        }
    }
    // not found
    None
}

fn get_user_method_setting_override(userid: u32) -> Option<PaymentMethod> {
    match env::var("USER_METHOD_SETTING_OVERRIDE").ok() {
        None => None,
        Some(s) => match get_user_method_setting_override_from_envstr(userid, &s) {
            None => None,
            Some(p) => Some(p),
        },
    }
}

/// Sanitize primary ID, such as:
/// - Replace "_" characters with "." (dot cannot appear in miner username, as it's the worker separator)
fn sanitize_primary_id(id: String) -> String {
    id.replace("_", ".")
}

/// Guess the payment method, adjust the primary ID
/// Return: payment method
fn guess_payment_method(orig_payment_id: &str) -> Result<PaymentMethod, Box<dyn Error>> {
    if orig_payment_id.contains(":") {
        // payment method prefix, e.g. "NOLN:"
        let prefix = orig_payment_id.split(":").next().unwrap_or_default();

        for pm in payment_methods() {
            if prefix == pm.to_string() {
                return Ok(*pm);
            }
        }

        // legacy "LA:" lightning address marker
        if prefix == "LA" {
            return Ok(PaymentMethod::PmLnAddress);
        }
    }
    // If it has '@', assume it is LA
    if orig_payment_id.contains("@") {
        return Ok(PaymentMethod::PmLnAddress);
    }
    // default: Nostr
    Ok(PaymentMethod::PmNostrLightning)
}

pub fn determine_payment_method(
    userid: u32,
    orig_payment_id: &str,
) -> Result<PaymentMethod, Box<dyn Error>> {
    if let Some(override_pm) = get_user_method_setting_override(userid) {
        println!(
            "Using payment method override {:?} for user {}",
            override_pm, userid
        );
        return Ok(override_pm);
    }
    // TODO if guessed is Nostr, check user setting from Nostr
    let guessed_pm = guess_payment_method(orig_payment_id)?;
    println!(
        "Using guessed payment method {:?} for user {}",
        guessed_pm, userid,
    );
    Ok(guessed_pm)
}

pub fn adjusted_primary_id(
    payment_method: PaymentMethod,
    orig_payment_id: &str,
) -> Result<String, Box<dyn Error>> {
    // remove prefix if present
    let without_prefix = if orig_payment_id.contains(":") {
        let prefix = orig_payment_id.split(":").next().unwrap_or_default();
        if prefix == payment_method.to_string() {
            // has a prefix, matching the payment method, remove it
            &orig_payment_id[(prefix.len() + 1)..]
        } else {
            // special case: legacy "LA:"
            if payment_method == PaymentMethod::PmLnAddress && prefix == "LA" {
                &orig_payment_id[3..]
            } else {
                // no change
                orig_payment_id
            }
        }
    } else {
        orig_payment_id
    };
    // Lightning address: character conversions
    if payment_method == PaymentMethod::PmLnAddress {
        let payment_id = sanitize_primary_id(without_prefix.to_string());
        return Ok(payment_id);
    }
    // no change
    Ok(without_prefix.to_owned())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_user_setting_override_from_envstr() {
        let s = "61:LNAD,62:NOLN,63:ZAP,72:NO_SUCH_PM";
        assert_eq!(
            get_user_method_setting_override_from_envstr(61, s),
            Some(PaymentMethod::PmLnAddress)
        );
        assert_eq!(
            get_user_method_setting_override_from_envstr(62, s),
            Some(PaymentMethod::PmNostrLightning)
        );
        assert_eq!(
            get_user_method_setting_override_from_envstr(63, s),
            Some(PaymentMethod::PmNostrZap)
        );
        // Not present:
        assert_eq!(get_user_method_setting_override_from_envstr(666, s), None);
        // Invalid PM
        assert_eq!(get_user_method_setting_override_from_envstr(72, s), None);
    }

    #[test]
    fn test_get_user_setting_override() {
        assert_eq!(get_user_method_setting_override(12), None);
        unsafe {
            env::set_var("USER_METHOD_SETTING_OVERRIDE", "661:LNAD,662:NOLN");
        }
        assert_eq!(
            get_user_method_setting_override(661),
            Some(PaymentMethod::PmLnAddress)
        );
        assert_eq!(
            get_user_method_setting_override(662),
            Some(PaymentMethod::PmNostrLightning)
        );
        assert_eq!(get_user_method_setting_override(666), None);
    }

    const NOSTR_ID1: &str = "npub12rv5lskctqxxs2c8rf2zlzc7xx3qpvzs3w4etgemauy9thegr43sf485vg";

    #[test]
    fn test_guess_payment_method() {
        {
            let s = NOSTR_ID1;
            let r = guess_payment_method(s).unwrap();
            assert_eq!(r, PaymentMethod::PmNostrLightning);
        }
        {
            let r = guess_payment_method("zappool@blink_sv").unwrap();
            assert_eq!(r, PaymentMethod::PmLnAddress);
        }
        {
            // payment method marker, 'LNAD:' lightning address
            let r = guess_payment_method("LNAD:zappool@blink_sv").unwrap();
            assert_eq!(r, PaymentMethod::PmLnAddress);
        }
        {
            // payment method marker, 'NOLN:' nostr lightning
            let r = guess_payment_method(&format!("NOLN:{}", NOSTR_ID1)).unwrap();
            assert_eq!(r, PaymentMethod::PmNostrLightning);
        }
        {
            // payment method marker, 'ZAP:' nostr zap
            let r = guess_payment_method(&format!("ZAP:{}", NOSTR_ID1)).unwrap();
            assert_eq!(r, PaymentMethod::PmNostrZap);
        }
        {
            // legacy lightning address "LA:" marker
            let r = guess_payment_method("LA:zappool@blink_sv").unwrap();
            assert_eq!(r, PaymentMethod::PmLnAddress);
        }
    }

    #[test]
    fn test_determine_payment_method() {
        // Cases with No override
        {
            let res = determine_payment_method(666, NOSTR_ID1).unwrap();
            assert_eq!(res, PaymentMethod::PmNostrLightning);
        }
        {
            let res = determine_payment_method(666, "LA:zappool@blink_sv").unwrap();
            assert_eq!(res, PaymentMethod::PmLnAddress);
        }
        // Cases with With override from env
        unsafe {
            env::set_var("USER_METHOD_SETTING_OVERRIDE", "661:LNAD,662:NOLN");
        }
        {
            let res = determine_payment_method(662, "LA:zappool@blink_sv").unwrap();
            assert_eq!(res, PaymentMethod::PmNostrLightning);
        }
    }

    #[test]
    fn test_adjusted_primary_id() {
        {
            let s = NOSTR_ID1;
            let r = adjusted_primary_id(PaymentMethod::PmNostrLightning, s).unwrap();
            assert_eq!(r, s);
        }
        {
            let r = adjusted_primary_id(PaymentMethod::PmLnAddress, "zappool@blink_sv").unwrap();
            assert_eq!(r, "zappool@blink.sv");
        }
        {
            let r = adjusted_primary_id(PaymentMethod::PmLnAddress, "LA:zappool@blink_sv").unwrap();
            assert_eq!(r, "zappool@blink.sv");
        }
        {
            let r = adjusted_primary_id(PaymentMethod::PmNostrLightning, "LA:zappool@blink_sv")
                .unwrap();
            assert_eq!(r, "LA:zappool@blink_sv");
        }
        {
            // with prefix
            let r =
                adjusted_primary_id(PaymentMethod::PmLnAddress, "LNAD:zappool@blink_sv").unwrap();
            assert_eq!(r, "zappool@blink.sv");
        }
        {
            // with prefix
            let r = adjusted_primary_id(
                PaymentMethod::PmNostrLightning,
                &format!("NOLN:{}", NOSTR_ID1),
            )
            .unwrap();
            assert_eq!(r, NOSTR_ID1);
        }
        {
            // with prefix
            let r = adjusted_primary_id(PaymentMethod::PmNostrZap, &format!("ZAP:{}", NOSTR_ID1))
                .unwrap();
            assert_eq!(r, NOSTR_ID1);
        }
    }
}
