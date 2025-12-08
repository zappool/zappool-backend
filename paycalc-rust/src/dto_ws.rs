use std::u32;

#[derive(Debug)]
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
    ) -> Self {
        Self {
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

    pub fn db_id(&self) -> u32 { self.db_id }
    pub fn time_add(&self) -> f32 { self.time_add }
}
