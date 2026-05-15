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
