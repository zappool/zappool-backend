from datetime import datetime, UTC

class Work:
    def __init__(self, db_id: int, uname_o: str, uname_o_wrkr: str, uname_u: str, uname_u_wrkr: str, tdiff: int, time_add: int, time_calc: int, calc_payout: int):
        self.db_id = db_id
        self.uname_o = uname_o
        self.uname_o_wrkr = uname_o_wrkr
        self.uname_u = uname_u
        self.uname_u_wrkr = uname_u_wrkr
        self.tdiff = tdiff
        self.time_add = time_add
        self.time_calc = time_calc
        self.calc_payout = calc_payout

    def new(uname_o: str, uname_u: str, tdiff: int):
        (uname_o, uname_o_wrkr) = Work.split_username_worker(uname_o)
        (uname_u, uname_u_wrkr) = Work.split_username_worker(uname_u)
        now_utc = datetime.now(UTC).timestamp()
        time_add = now_utc
        time_calc = 0
        calc_payout = 0
        return Work(0, uname_o, uname_o_wrkr, uname_u, uname_u_wrkr, tdiff, time_add, time_calc, calc_payout)

    def split_username_worker(full_username: str) -> (str, str):
        dotindex = full_username.find(".")
        if dotindex < 0:
            return (full_username, "")
        return (full_username[:dotindex], full_username[dotindex+1:])

