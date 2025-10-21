from db_oc import db_setup_1

import sys
sys.path.insert(1, "common")
from common_main import get_db_file

import sqlite3


dbfile = get_db_file("ocean.db", create_mode=True)

print(f"Initialize DB {dbfile}. Press Y to continue")
input = input()
if input.upper() != "Y":
    print(f"Aborting")
    sys.exit(1)

# Connect to SQLite database
conn = sqlite3.connect(dbfile)
db_setup_1(conn)
conn.close()

print(f"New empty db created, don't forget to rename! {dbfile}")
