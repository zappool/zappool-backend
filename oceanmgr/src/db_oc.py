from dto_oc import BlockEarning
from ocean_web import EarningSnapshot

import sqlite3
from datetime import datetime

def db_setup_1(conn: sqlite3.Connection):
    cursor = conn.cursor()

    # Create table
    cursor.execute("""
        CREATE TABLE OC_BLOCK_EARN
        (Time INTEGER, BlockHash VARCHAR(100), Earning INTEGER, PoolFee INTEGER, TimeAddedFirst INTEGER, TimeUpdated INTEGER)
    """)
    cursor.execute("CREATE INDEX OcBlockEarnTime ON OC_BLOCK_EARN (Time)")

    cursor.execute("CREATE TABLE OC_EARN (Time INTEGER, Estimated INTEGER, AcctdUnpaid INTEGER, AcctdPaid INTEGER)")
    cursor.execute("CREATE INDEX OcEarnTime ON OC_EARN (Time)")

    # Commit changes and close connection
    conn.commit()


def insert_or_update_block_earning(cursor, earning: BlockEarning, now: datetime):
    cursor.execute("SELECT COUNT(*) FROM OC_BLOCK_EARN WHERE BlockHash = ?", (earning.block_hash,))
    rows = cursor.fetchall()
    if len(rows) >= 1:
        if len(rows[0]) >= 1:
            if rows[0][0] >= 1:
                # already present, update
                cursor.execute(
                    "UPDATE OC_BLOCK_EARN SET Time = ?, Earning = ?, PoolFee = ?, TimeUpdated = ? WHERE BlockHash = ?",
                    (earning.time, earning.earned_sats, earning.pool_fee, now, earning.block_hash,)
                )
                # print("updated")
                return
    # not present, insert
    cursor.execute("""
        INSERT INTO OC_BLOCK_EARN
        (Time, BlockHash, Earning, PoolFee, TimeAddedFirst, TimeUpdated)
        VALUES (?, ?, ?, ?, ?, ?)
    """, (earning.time, earning.block_hash, earning.earned_sats, earning.pool_fee, now, now,))
    # print("inserted")

def block_earnings_count(cursor) -> int:
    cursor.execute("SELECT COUNT(*) FROM OC_BLOCK_EARN")
    rows = cursor.fetchall()
    if len(rows) >= 1:
        if len(rows[0]) >= 1:
            return rows[0][0]
    return 0

def block_earnings_sum(cursor) -> int:
    cursor.execute("SELECT SUM(Earning) FROM OC_BLOCK_EARN")
    rows = cursor.fetchall()
    if len(rows) >= 1:
        if len(rows[0]) >= 1:
            return rows[0][0]
    return 0

def get_last_block(cursor: sqlite3.Cursor) -> BlockEarning:
    cursor.execute("""
        SELECT
        Time, BlockHash, Earning, PoolFee, TimeAddedFirst, TimeUpdated
        FROM OC_BLOCK_EARN
        ORDER BY Time DESC
        LIMIT 1
    """)
    rows = cursor.fetchall()
    if len(rows) >= 1:
        row = rows[0]
        if len(row) >= 4:
            return BlockEarning(row[0], row[1], row[2], row[3])
    return None

# Get the blocks after a certain time, oldest first.
# Old time is typically the time of the already processed last block.
def get_new_blocks(cursor: sqlite3.Cursor, old_time: int) -> list[BlockEarning]:
    cursor.execute("""
        SELECT
        Time, BlockHash, Earning, PoolFee, TimeAddedFirst, TimeUpdated
        FROM OC_BLOCK_EARN
        WHERE Time > ?
        ORDER BY Time ASC
    """, (old_time,))
    rows = cursor.fetchall()
    res = []
    for row in rows:
        if len(row) >= 4:
            res.append(BlockEarning(row[0], row[1], row[2], row[3]))
    return res


# Count the blocks after a certain time, oldest first.
# Old time is typically the time of the already processed last block.
def count_new_blocks(cursor: sqlite3.Cursor, old_time: int) -> int:
    cursor.execute("""
        SELECT COUNT(*)
        FROM OC_BLOCK_EARN
        WHERE Time > ?
    """, (old_time,))
    rows = cursor.fetchall()
    if len(rows) >= 1:
        if len(rows[0]) >= 1:
            return rows[0][0]
    return 0


def insert_earn_snapshot(conn, earns: EarningSnapshot):
    cursor = conn.cursor()

    cursor.execute(
        """
            INSERT INTO OC_EARN 
                (Time, Estimated, AcctdUnpaid, AcctdPaid)
            VALUES
                (?, ?, ?, ?)
        """,
        (earns.time, earns.estimated, earns.accounted_unpaid, earns.accounted_paid,)
    )

    cursor.close()
    conn.commit()

def get_last_snapshot(cursor) -> EarningSnapshot:
    cursor.execute("SELECT Time, Estimated, AcctdUnpaid, AcctdPaid FROM OC_EARN ORDER BY Time DESC")
    rows = cursor.fetchall()
    if len(rows) < 1:
        return None
    if len(rows[0]) < 4:
        return None
    return EarningSnapshot(int(rows[0][0]), int(rows[0][3]), int(rows[0][2]), int(rows[0][1]))


def get_last_snapshot_before(cursor, before_time: int) -> EarningSnapshot:
    cursor.execute("SELECT Time, Estimated, AcctdUnpaid, AcctdPaid FROM OC_EARN WHERE Time < ? ORDER BY Time DESC", (before_time,))
    rows = cursor.fetchall()
    if len(rows) < 1:
        return None
    if len(rows[0]) < 4:
        return None
    return EarningSnapshot(int(rows[0][0]), int(rows[0][3]), int(rows[0][2]), int(rows[0][1]))


def get_all_snapshots(cursor) -> dict[int, int]:
    cursor.execute("SELECT Time, Estimated, AcctdUnpaid, AcctdPaid FROM OC_EARN ORDER BY Time DESC")
    rows = cursor.fetchall()
    res = {}
    for row in rows:
        if len(row) >= 4:
            time = int(row[0])
            paid = int(row[3])
            total = paid + int(row[2]) + int(row[1])
            # print(time, total)
            res[time] = [total, paid]
    return res

