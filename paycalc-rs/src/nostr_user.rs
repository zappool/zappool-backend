/// Nostr user settings
use crate::common::PaymentMethod;

use std::env;
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

pub fn get_user_method_setting_override(userid: u32) -> Option<PaymentMethod> {
    match env::var("USER_METHOD_SETTING_OVERRIDE").ok() {
        None => None,
        Some(s) => match get_user_method_setting_override_from_envstr(userid, &s) {
            None => None,
            Some(p) => {
                println!("Using payment method override {:?} for user {}", p, userid);
                Some(p)
            }
        },
    }
}

#[cfg(test)]
mod test {
    use crate::common::PaymentMethod;

    use super::*;

    #[test]
    fn test_get_user_setting_override_from_envstr() {
        let s = "12:LNAD,4:NOLN,72:NO_SUCH_PM";
        assert_eq!(
            get_user_method_setting_override_from_envstr(12, s),
            Some(PaymentMethod::PmLnAddress)
        );
        assert_eq!(
            get_user_method_setting_override_from_envstr(4, s),
            Some(PaymentMethod::PmNostrLightning)
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
    }
}
