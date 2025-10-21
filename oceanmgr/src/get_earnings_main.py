import ocean

import sys
sys.path.insert(1, "common")
from common_main import get_db_file

from dotenv import load_dotenv
import os


load_dotenv()

# accessing and printing value
mother_pool_user = os.getenv("MOTHER_POOL_USER")
print(f"Mother pool user: {mother_pool_user}")
if len(mother_pool_user) < 10:
    raise Exception(f"Missing mother pool user, set it in .env! ({mother_pool_user})")

#test_user = "bc1q98wufxmtfh5qlk7fe5dzy2z8cflvqjysrh4fx2"

dbfile = get_db_file("ocean.db")

# Start with printing the current snaphost, to check DB, etc
ocean.print_current_earnings(dbfile)

ocean.get_earnings_loop(dbfile, mother_pool_user)


