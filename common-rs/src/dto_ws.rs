use std::u32;

#[derive(Debug)]
pub struct Work {
    pub db_id: u32,
    pub uname_o: String,
    pub uname_o_wrkr: String,
    pub uname_u: String,
    pub uname_u_wrkr: String,
    pub tdiff: u32,
    pub time_add: f64,
    pub time_calc: u32,
    pub calc_payout: u32,
}

impl Work {
    pub fn new(
        db_id: u32,
        uname_o: String,
        uname_o_wrkr: String,
        uname_u: String,
        uname_u_wrkr: String,
        tdiff: u32,
        time_add: f64,
        time_calc: u32,
        calc_payout: u32,
    ) -> Self {
        Self {
            db_id,
            uname_o,
            uname_o_wrkr,
            uname_u,
            uname_u_wrkr,
            tdiff,
            time_add,
            time_calc,
            calc_payout,
        }
    }
}
