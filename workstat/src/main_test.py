import db_ws as db

import sys
sys.path.insert(1, "common")
from common_main import get_db_file

import sqlite3


dbfile = get_db_file("workstat.db")

conn = sqlite3.connect(dbfile)

# Insert some records
db.insert_work(conn, "lnblabla.worker1", "bc1something.123abc", 131072)
db.insert_work(conn, "lnblabla.worker2", "bc1something.321cba", 131072)
db.insert_work(conn, "lnblabla.worker1", "bc1something.123abc", 131072)
db.insert_work(conn, "lnblabla.worker1", "bc1something.123abc", 131072)

# Fetch and display records
cursor = conn.cursor()
res = db.get_all_work_limit(cursor, 1000)

for w in res:
    print(w.db_id, w.uname_o, w.uname_o_wrkr, w.uname_u, w.uname_u_wrkr, w.tdiff, w.time_add, w.time_calc, w.calc_payout)

