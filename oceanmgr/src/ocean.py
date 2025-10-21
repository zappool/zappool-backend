import db_oc as db
import ocean_web

from datetime import datetime, UTC
import sqlite3
import time


def get_and_save_block_earnings(dbfile: str, ocean_account: str):
    earns = ocean_web.get_block_earnings(ocean_account)

    conn = sqlite3.connect(dbfile)
    cursor = conn.cursor()

    now_utc = datetime.now(UTC).timestamp()

    cnt = 0
    try:
        for e in earns:
            db.insert_or_update_block_earning(cursor, e, now_utc)
            cnt += 1
        conn.commit()

    finally:
        cursor.close()
        conn.close()
    return cnt


def get_and_save_earning_snapshot(dbfile: str, ocean_account: str):
    earn = ocean_web.get_earning_snapshot(ocean_account)

    conn = sqlite3.connect(dbfile)

    try:
        db.insert_earn_snapshot(conn, earn)
        conn.commit()

    finally:
        conn.close()


def print_current_earnings(dbfile: str):
    conn = sqlite3.connect(dbfile)
    cursor = conn.cursor()

    try:
        block_earn_cnt = db.block_earnings_count(cursor)
        block_earn_sum = db.block_earnings_sum(cursor)
        last_block = db.get_last_block(cursor)
        last_block_time = 0
        if last_block != None:
            last_block_time = last_block.time
        print(f"Sum block earnings: {block_earn_sum} (in {block_earn_cnt} blocks, last block {last_block_time})")


        snap = db.get_last_snapshot(cursor)
        if not snap:
            print(f"No current snapshot could be read, db {dbfile}")
        else:
            print("Current latest snapshot:")
            print(snap.to_string())
    except Exception as ex:
        print(f"Exception: {ex}")
    finally:
        cursor.close()
        conn.close()


def dump_snapshots(dbfile: str):
    conn = sqlite3.connect(dbfile)
    cursor = conn.cursor()

    try:
        res = db.get_all_snapshots(cursor)
        for t in res:
            print(f"{t}: {res[t][0]} {res[t][1]}")
    except Exception as ex:
        print(f"Exception: {ex}")
    finally:
        conn.close()


def get_earnings_loop(dbfile: str, ocean_account: str):
    sleep_secs = 10 * 60
    next_time = time.time()

    while True:
        res_cnt = 0
        try:
            res_cnt = get_and_save_block_earnings(dbfile, ocean_account)
        except Exception as ex:
            print(f"Exception: get_earnings_loop: {ex}")
            continue
        print(f"Current block earnings saved ({res_cnt})")

        try:
            get_and_save_earning_snapshot(dbfile, ocean_account)
        except Exception as ex:
            print(f"Exception: get_earnings_loop: {ex}")
            continue
        print("Current earnings snapshot saved")

        print_current_earnings(dbfile)

        next_time = next_time + sleep_secs
        to_sleep = max(next_time - time.time(), 1)
        if to_sleep > 0:
            print(f"Sleeping for {round(to_sleep)} secs... (next_time {round(next_time)})")
            time.sleep(to_sleep)

