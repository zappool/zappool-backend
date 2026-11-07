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

#[cfg(test)]
mod tests {
    use super::{shorten_id, shorten_id_m_n};

    #[test]
    fn test_shorten_id() {
        assert_eq!(
            shorten_id("This_is_a_quite+but+not+extremly-long=string"),
            "This_is_a..ring"
        );
        assert_eq!(shorten_id("small"), "small");
        assert_eq!(shorten_id("1"), "1");
        assert_eq!(shorten_id("0"), "0");
        assert_eq!(shorten_id("123456789012"), "123456789012");
        assert_eq!(shorten_id("1234567890123"), "1234567890123");
        assert_eq!(shorten_id("12345678901234"), "12345678901234");
        assert_eq!(shorten_id("123456789012345"), "123456789012345");
        assert_eq!(shorten_id("1234567890123456"), "123456789..3456");
    }

    #[test]
    fn test_shorten_id_m_n() {
        assert_eq!(
            shorten_id_m_n("This_is_a_quite+but+not+extremly-long=string", 9, 4),
            "This_is_a..ring"
        );
        assert_eq!(
            shorten_id_m_n("This_is_a_quite+but+not+extremly-long=string", 9, 1),
            "This_is_a..g"
        );
        assert_eq!(
            shorten_id_m_n("This_is_a_quite+but+not+extremly-long=string", 4, 4),
            "This..ring"
        );
    }
}
