use chrono::DateTime;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub struct Work {
    pub db_id: u32,
    pub uname_o: String,
    pub uname_o_wrkr: String,
    pub uname_u: String,
    pub uname_u_wrkr: String,
    pub uname_o_id: u32,
    pub uname_o_wrkr_id: u32,
    pub uname_u_id: u32,
    pub uname_u_wrkr_id: u32,
    pub tdiff: u32,
    pub time_add: f64,
    pub payed: u64,
    pub payed_time: u32,
    pub payed_ref: String,
    pub committed: u64,
    pub commit_blocks: u16,
    pub commit_first_time: u32,
    pub commit_next_time: u32,
    pub estimate: u64, // Msats!
}

impl Work {
    pub fn new(
        db_id: u32,
        uname_o: String,
        uname_o_wrkr: String,
        uname_u: String,
        uname_u_wrkr: String,
        uname_o_id: u32,
        uname_o_wrkr_id: u32,
        uname_u_id: u32,
        uname_u_wrkr_id: u32,
        tdiff: u32,
        time_add: f64,
        payed: u64,
        payed_time: u32,
        payed_ref: String,
        committed: u64,
        commit_blocks: u16,
        commit_first_time: u32,
        commit_next_time: u32,
        estimate_msats: u64,
    ) -> Self {
        Self {
            db_id,
            uname_o,
            uname_o_wrkr,
            uname_u,
            uname_u_wrkr,
            uname_o_id,
            uname_o_wrkr_id,
            uname_u_id,
            uname_u_wrkr_id,
            tdiff,
            time_add,
            payed,
            payed_time,
            payed_ref,
            committed,
            commit_blocks,
            commit_first_time,
            commit_next_time,
            estimate: estimate_msats,
        }
    }

    pub fn new_with_diff(uname_o: &str, uname_u: &str, tdiff: u32) -> Self {
        let (uname_o, uname_o_wrkr) = Self::split_username_worker(uname_o);
        let (uname_u, uname_u_wrkr) = Self::split_username_worker(uname_u);
        let now_utc = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        let time_add = now_utc;
        Self::new(
            0,
            uname_o,
            uname_o_wrkr,
            uname_u,
            uname_u_wrkr,
            0,
            0,
            0,
            0,
            tdiff,
            time_add,
            0,
            0,
            "".to_string(),
            0,
            0,
            0,
            0,
            0,
        )
    }

    pub fn split_username_worker(full_username: &str) -> (String, String) {
        match full_username.find(".") {
            None => (full_username.to_string(), "".to_string()),
            Some(dotindex) => (
                full_username[..dotindex].to_string(),
                full_username[dotindex + 1..].to_string(),
            ),
        }
    }
}

// Block earning: a piece of earned earning, connected to a block found
pub struct Block {
    pub time: u32,
    pub block_hash: String,
    pub earned_sats: u64,
    pub pool_fee: u32,
    pub acc_total_diff: u64,
}

impl Block {
    pub fn new(
        time: u32,
        block_hash: String,
        earned_sats: u64,
        pool_fee: u32,
        acc_total_diff: u64,
    ) -> Self {
        Self {
            time,
            block_hash,
            earned_sats,
            pool_fee,
            acc_total_diff,
        }
    }
}

impl ToString for Block {
    fn to_string(&self) -> String {
        let t = DateTime::from_timestamp(self.time as i64, 0).unwrap_or_default();
        format!(
            "{} {} {} {} {}",
            t.to_string(),
            self.block_hash,
            self.earned_sats,
            self.pool_fee,
            self.acc_total_diff
        )
    }
}

// A snapshot a miner at a given time.
// Miner is the base miner username (without worker name)
// Amounts in Msat
#[derive(Clone)]
pub struct MinerSnapshot {
    pub user_id: u32,
    pub user_s: String,
    pub time: u32,
    pub tot_commit: u64,
    pub tot_estimate: u64,
    pub tot_paid: u64,
    // Unpaid, diff bewteen estimate and paid; Msat
    pub unpaid: u64,
    // Diff between conservative estimate (committed + most estimate) and paid, may be negative; Msat
    pub unpaid_cons: u64,
    pub payreq_id: i32,
}

impl MinerSnapshot {
    pub fn new(
        user_id: u32,
        user_s: String,
        time: u32,
        tot_commit: u64,
        tot_estimate: u64,
        tot_paid: u64,
        unpaid: u64,
        unpaid_cons: u64,
        payreq_id: i32,
    ) -> Self {
        Self {
            user_id,
            user_s,
            time,
            tot_commit,
            tot_estimate,
            tot_paid,
            unpaid,
            unpaid_cons,
            payreq_id,
        }
    }
}

#[derive(Clone)]
pub struct PayRequest {
    pub id: i32,
    pub miner_id: u32,
    pub req_amnt: u64,
    pub pay_method: String,
    pub pri_id: String,
    pub req_time: u32,
}

impl PayRequest {
    pub fn new(
        id: i32,
        miner_id: u32,
        req_amnt: u64,
        pay_method: String,
        pri_id: String,
        req_time: u32,
    ) -> Self {
        Self {
            id,
            miner_id,
            req_amnt,
            pay_method,
            pri_id,
            req_time,
        }
    }
}

pub struct Payment {
    pub id: i32,
    pub req_id: i32,
    pub create_time: u32,
    pub status: u8,
    pub status_time: u32,
    pub error_code: u8,
    pub error_str: String,
    pub retry_cnt: u8,
    pub fail_time: u32,
    pub secon_id: String,
    pub terti_id: String,
    pub paid_amnt: u64,
    pub paid_fee: u32,
    pub pay_time: u32,
    pub pay_ref: String,
}

impl Payment {
    pub fn new(
        id: i32,
        req_id: i32,
        create_time: u32,
        status: u8,
        status_time: u32,
        error_code: u8,
        error_str: String,
        retry_cnt: u8,
        fail_time: u32,
        secon_id: String,
        terti_id: String,
        paid_amnt: u64,
        paid_fee: u32,
        pay_time: u32,
        pay_ref: String,
    ) -> Self {
        Self {
            id,
            req_id,
            create_time,
            status,
            status_time,
            error_code,
            error_str,
            retry_cnt,
            fail_time,
            secon_id,
            terti_id,
            paid_amnt,
            paid_fee,
            pay_time,
            pay_ref,
        }
    }
}
