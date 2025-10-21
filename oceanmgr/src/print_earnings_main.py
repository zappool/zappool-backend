import ocean

import sys
sys.path.insert(1, "common")
from common_main import get_db_file

import sys


dbfile = get_db_file("ocean.db")

# Start with printing the current snaphost, to check DB, etc
ocean.print_current_earnings(dbfile)

ocean.dump_snapshots(dbfile)


