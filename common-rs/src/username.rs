/// Username handling

/// Map usernames, from the original full username to upstream full username
///  npub12gygh77v0ux4xk73vvht34lf3g8hs3vfsdjs823ts6pce9n28ehq8edvt8             -->    bc1q98wufxmtfh5qlk7fe5dzy2z8cflvqjysrh4fx2
///  npub12gygh77v0ux4xk73vvht34lf3g8hs3vfsdjs823ts6pce9n28ehq8edvt8.bitaxe      -->    bc1q98wufxmtfh5qlk7fe5dzy2z8cflvqjysrh4fx2.8c65fb71
pub fn map_full_username(orig_full_user: &str, upstream_user: &str) -> String {
    let (_orig_user, orig_worker) = split_full_username(orig_full_user);
    if orig_worker.is_empty() {
        upstream_user.to_string()
    } else {
        let hash = hash_username(orig_full_user);
        format!("{upstream_user}.{hash}")
    }
}

/// Split full username to username and worker
pub fn split_full_username(full_username: &str) -> (String, String) {
    match full_username.find('.') {
        None => (full_username.to_string(), String::new()),
        Some(dot_idx) => (
            full_username[..dot_idx].to_string(),
            full_username[dot_idx + 1..].to_string(),
        ),
    }
}

/// Hash the full username, to derive upstream worker. Use SHA256
pub fn hash_username(full_username: &str) -> String {
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(full_username.as_bytes());
    hash.iter().take(4).map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use crate::username::{hash_username, map_full_username, split_full_username};

    #[test]
    fn test_map_full_username() {
        let upstream_user = "bc1q98wufxmtfh5qlk7fe5dzy2z8cflvqjysrh4fx2";

        assert_eq!(map_full_username("user1", upstream_user), upstream_user);
        assert_eq!(
            map_full_username("user1.worker1", upstream_user),
            "bc1q98wufxmtfh5qlk7fe5dzy2z8cflvqjysrh4fx2.33043d99"
        );
        assert_eq!(
            map_full_username("user1.worker2", upstream_user),
            "bc1q98wufxmtfh5qlk7fe5dzy2z8cflvqjysrh4fx2.bca10bf6"
        );
        assert_eq!(
            map_full_username(
                "npub12gygh77v0ux4xk73vvht34lf3g8hs3vfsdjs823ts6pce9n28ehq8edvt8",
                upstream_user
            ),
            upstream_user
        );
        assert_eq!(
            map_full_username(
                "npub12gygh77v0ux4xk73vvht34lf3g8hs3vfsdjs823ts6pce9n28ehq8edvt8.bitaxe",
                upstream_user
            ),
            "bc1q98wufxmtfh5qlk7fe5dzy2z8cflvqjysrh4fx2.db2b87b2"
        );
    }

    #[test]
    fn test_split_username_worker() {
        assert_eq!(
            split_full_username("user1"),
            ("user1".to_string(), "".to_string())
        );
        assert_eq!(
            split_full_username("user1.worker1"),
            ("user1".to_string(), "worker1".to_string())
        );
        assert_eq!(
            split_full_username("1.2"),
            ("1".to_string(), "2".to_string())
        );
        assert_eq!(
            split_full_username("1.2.3"),
            ("1".to_string(), "2.3".to_string())
        );
        assert_eq!(split_full_username("1"), ("1".to_string(), "".to_string()));
        assert_eq!(split_full_username(""), ("".to_string(), "".to_string()));
    }

    #[test]
    fn test_hash_username() {
        assert_eq!(
            hash_username("npub12gygh77v0ux4xk73vvht34lf3g8hs3vfsdjs823ts6pce9n28ehq8edvt8.bitaxe"),
            "db2b87b2", // "8c65fb71" ?
        );
        assert_eq!(
            hash_username(
                "npub1s0kz0frwwyx69z83fpzulj90jdy5804phwglu2p2ah95hf2p9czsscpfpw.bitaxesupra"
            ),
            "be64d4d2",
        );
    }
}
