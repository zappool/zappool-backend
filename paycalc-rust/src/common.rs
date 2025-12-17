/// Payment methods:
pub struct PaymentMethod {}

impl PaymentMethod {
    /// Lightning Address: Lightning Address -> Lightning payment
    pub const PAYMENT_METHOD_LN_ADDRESS: &str = "LNAD";
    /// Nostr Lightning: NPub -> Nostr Profile -> Lightning Address -> Lightning payment
    pub const PAYMENT_METHOD_NOSTR_LIGHTNING: &str = "NOLN";
}

pub struct PaymentResult {
    pub success: bool,
    pub err_nonfinal: bool,
    pub err_code: u8,
    pub err_str: String,
    pub secon_id: String,
    pub terti_id: String,
    pub paid_amount: u64,
    pub paid_fee: u32,
    pub reference: String,
}

impl PaymentResult {
    pub fn new(
        success: bool,
        err_nonfinal: bool,
        err_code: u8,
        err_str: &str,
        secon_id: &str,
        terti_id: &str,
        paid_amount: u64,
        paid_fee: u32,
        reference: &str,
    ) -> Self {
        Self {
            success,
            err_nonfinal,
            err_code,
            err_str: err_str.to_string(),
            secon_id: secon_id.to_string(),
            terti_id: terti_id.to_string(),
            paid_amount,
            paid_fee,
            reference: reference.to_string(),
        }
    }
}

// Shorten a string ID by leaving out the middle, for printing
// E.g.: shorten_id_m_n("npub1xseyc0xgytdu0mdua7gc540reyzlu98n7rcvlz7p3kc6txlauzfqmemekt", 9, 4)
//  --> "npub1xsey..mekt"
pub fn shorten_id_m_n(id: &str, prefix_len: u16, postfix_len: u16) -> String {
    let prefix_len = prefix_len as usize;
    let postfix_len = postfix_len as usize;
    let l = prefix_len + 2 + postfix_len;
    if id.len() <= l {
        id.to_string()
    } else {
        format!("{}..{}", &id[..prefix_len], &id[(id.len() - postfix_len)..])
    }
}

// Shorten a string ID by leaving out the middle, for printing
pub fn shorten_id(id: &str) -> String {
    shorten_id_m_n(id, 9, 4)
}
