use std::u32;

#[derive(Debug)]
pub struct BlockEarning {
    pub time: u32,
    pub block_hash: String,
    earned_sats: u64,
    pool_fee: u32,
}

impl BlockEarning {
    pub fn new(
        time: u32,
        block_hash: String,
        earned_sats: u64,
        pool_fee: u32,
    ) -> Self {
        Self {
            time,
            block_hash,
            earned_sats,
            pool_fee,
        }
    }
}
