import db_ws as db

import sys
sys.path.insert(1, "common")
from common_main import get_db_file

import sqlite3


dbfile = get_db_file("workstat.db")
dbfile_uri_ro = "file:" + dbfile + "?mode=ro"
# print(dbfile_uri_ro)
conn = sqlite3.connect(dbfile_uri_ro, uri=True)

# Fetch and display records
cursor = conn.cursor()
res = db.get_all_work_limit(cursor, 100)

print(f"({len(res)}):")
for w in res:
    print(w.db_id, w.uname_o, w.uname_o_wrkr, w.uname_u, w.uname_u_wrkr, w.tdiff, w.time_add, w.time_calc, w.calc_payout)

