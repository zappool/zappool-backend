// use std::time::{SystemTime, UNIX_EPOCH};
use std::u32;

pub struct Work {
    db_id: u32,
    uname_o: String,
    uname_o_wrkr: String,
    uname_u: String,
    uname_u_wrkr: String,
    tdiff: u32,
    time_add: f32,
    time_calc: u32,
    calc_payout: u32,
}

impl Work {
    pub fn new(
        db_id: u32,
        uname_o: &str,
        uname_o_wrkr: &str,
        uname_u: &str,
        uname_u_wrkr: &str,
        tdiff: u32,
        time_add: f32,
        time_calc: u32,
        calc_payout: u32,
    ) -> Work {
        Work {
            db_id,
            uname_o: uname_o.to_string(),
            uname_o_wrkr: uname_o_wrkr.to_string(),
            uname_u: uname_u.to_string(),
            uname_u_wrkr: uname_u_wrkr.to_string(),
            tdiff,
            time_add,
            time_calc,
            calc_payout,
        }
    }

    /*
    fn create(
        uname_o_full: &str,
        uname_u_full: &str,
        tdiff: u32,
    ) -> Work {
        let (uname_o, uname_o_wrkr) = Self::split_username_worker(uname_o_full);
        let (uname_u, uname_u_wrkr) = Self::split_username_worker(uname_u_full);
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32;

        Work {
            db_id: 0,
            uname_o,
            uname_o_wrkr,
            uname_u,
            uname_u_wrkr,
            tdiff,
            time_add: now as f32,
            time_calc: 0,
            calc_payout: 0,
        }
    }
    */

    pub fn db_id(&self) -> u32 { self.db_id }
    pub fn time_add(&self) -> f32 { self.time_add }

    /*
    fn split_username_worker(full_username: &str) -> (String, String) {
        match full_username.find(".") {
            None => (full_username.to_string(), "".to_string()),
            Some(dotindex) => (full_username[..dotindex].to_string(), full_username[dotindex+1..].to_string()),
        }
    }
    */
}
