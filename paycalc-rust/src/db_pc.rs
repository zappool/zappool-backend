use crate::common::get_db_update_versions_from_args;
use crate::dto_pc::{Block, Work};

use rusqlite::{Connection, Row, Transaction};
use std::error::Error;

// static BLOCKS_WINDOW: u8 = 8;
pub static LATEST_DB_VERSION: u8 = 3;

// Upgrade from an older version, versions taken from args
pub fn db_setup(conn: &Connection) -> Result<(), Box<dyn Error>> {
    let (vto, vfrom) = get_db_update_versions_from_args(LATEST_DB_VERSION);
    db_setup_from_to(conn, Some(vfrom), Some(vto))
}

// Upgrade from an older version, versions have default values
fn db_setup_from_to(conn: &Connection, vfrom: Option<u8>, vto: Option<u8>) -> Result<(), Box<dyn Error>> {
    let vfrom = vfrom.unwrap_or(0);
    let vto = vto.unwrap_or(LATEST_DB_VERSION);
    println!("Updating DB from v{vfrom} to v{vto}");

    if vfrom <= 0 && vto >= 3 {
        db_update_0_3(conn)?;
    }

    Ok(())
}

// Note: v3 is used to be at par with original prototype impl (python)
fn db_update_0_3(conn: &Connection) -> Result<(), Box<dyn Error>> {
    let _ = conn.execute("CREATE TABLE VERSION (Version INTEGER)", [])?;
    let _ = conn.execute("INSERT INTO VERSION (Version) VALUES (3)", [])?;

    let _ = conn.execute("UPDATE VERSION SET Version = 2", [])?;

    let _ = conn.execute(
        "CREATE TABLE STATUS ( \
            LastWorkItemRetrvd INTEGER, \
            LastWorkItemTimeRetrvd INTEGER, \
            LastBlockRetrvd INTEGER, \
            LastBlockProcd INTEGER, \
            LastPaymentProcd INTEGER)",
        [])?;
    let _ = conn.execute(
        "INSERT INTO STATUS \
                (LastWorkItemRetrvd, LastWorkItemTimeRetrvd, LastBlockRetrvd, LastBlockProcd, LastPaymentProcd) \
            VALUES \
                (-1, 0, 0, 0, 0)",
        [])?;

    // Create table USERLOOKUP, lookup table for user strings (original, upstream, worker, all)
    let _ = conn.execute(
        "CREATE TABLE USERLOOKUP \
            (Id INTEGER PRIMARY KEY AUTOINCREMENT, String VARCHAR(100), Type INTEGER, TimeAdd INTEGER)",
        [])?;

    let _ = conn.execute("CREATE INDEX UserlookupId ON USERLOOKUP (Id)", [])?;
    let _ = conn.execute("CREATE INDEX UserlookupString ON USERLOOKUP (String)", [])?;

    // Create table WORK
    // UNameO -- original username id (without worker name)
    // UNameOWrkr - Optional worker in the original username id
    // UNameU -- upstream username id (without worker name)
    // UNameUWrkr - Worker in the upstream username id
    // TDiff - The target diffuclty of the work request
    // TimeAdd - Time when package was added
    // Payed - Amount already paid
    // PayedTime - Time when payout was completed
    // PayedRef - Reference of completed payment
    // Committed - Amount committed, in Msat
    // CommitBlocks - Number of blocks that are the source of commitments, 0 or 1--7 or 8
    // CommitFirstTime - Time when commitment from first block was done
    // CommitNextTime - Time when commitment from the current last block was done
    // Estimate - Estimate for the unaccounted blocks in Msat (in addition to committed)
    // // TimeCalc - Time when payout was calculated
    // // CalcPayout - The calculated estimated payout, sats
    let _ = conn.execute(
        "CREATE TABLE WORK ( \
            Id INTEGER PRIMARY KEY AUTOINCREMENT, \
            UNameO INTEGER, \
            UNameOWrkr INTEGER, \
            UNameU INTEGER, \
            UNameUWrkr INTEGER, \
            TDiff INTEGER, \
            TimeAdd INTEGER, \
            Payed INTEGER, \
            PayedTime INTEGER, \
            PayedRef VARCHAR(500), \
            Committed INTEGER, \
            CommitBlocks INTEGER, \
            CommitFirstTime INTEGER, \
            CommitNextTime INTEGER, \
            Estimate INTEGER, \
            FOREIGN KEY (UNameO) REFERENCES USERLOOKUP(Id), \
            FOREIGN KEY (UNameOWrkr) REFERENCES USERLOOKUP(Id), \
            FOREIGN KEY (UNameU) REFERENCES USERLOOKUP(Id) \
            FOREIGN KEY (UNameUWrkr) REFERENCES USERLOOKUP(Id))",
        [])?;
    let _ = conn.execute("CREATE INDEX WorkTimeAdd ON WORK (TimeAdd)", [])?;

    let _ = conn.execute(
        "CREATE TABLE PC_BLOCK ( \
            Time INTEGER, \
            BlockHash VARCHAR(100), \
            Earning INTEGER, \
            PoolFee INTEGER, \
            TimeAddedFirst INTEGER, \
            TimeUpdated INTEGER, \
            AccTotalDiff INTEGER)",
        [])?;
    let _ = conn.execute("CREATE INDEX OcBlockEarnTime ON PC_BLOCK (Time)", [])?;

    // Create table MINER_SS (Snapshot)
    // At most one record per miner
    // UserId -- the base miner username (without worker name) id
    // User -- the base miner username (without worker name)
    // Time -- Current time when shapshot is valid
    // TotCommit -- Total amount committed in all workitems, Msat
    // TotEstimate -- Total amount estimated (incl. committed) in all workitems, Msat
    // TotPaid -- Total amount paid, Msat
    // Unpaid -- Unpaid, diff bewteen full estimate and paid; Msat
    // UnpaidCons -- Diff between conservative estimate (committed + most estimate) and paid, may be negative; Msat
    // PayReqId -- Id of the payrequest, if a payment is currently scheduled
    let _ = conn.execute(
        "CREATE TABLE MINER_SS ( \
            UserId INTEGER PRIMARY KEY, \
            UserS VARCHAR(100), \
            Time INTEGER, \
            TotCommit INTEGER, \
            TotEstimate INTEGER, \
            TotPaid INTEGER, \
            Unpaid INTEGER, \
            UnpaidCons INTEGER, \
            PayReqId INTEGER)",
            [])?;
    let _ = conn.execute("CREATE INDEX MinerSSUserId ON MINER_SS (UserId)", [])?;
    let _ = conn.execute("CREATE INDEX MinerSSUserS ON MINER_SS (UserS)", [])?;

    // Create table MINER_SS_HIST (Historical Snapshots)
    // Columns mostly as in MINER_SS
    let _ = conn.execute(
        "CREATE TABLE MINER_SS_HIST ( \
            UserId VARCHAR(100), \
            Time INTEGER, \
            TotCommit INTEGER, \
            TotEstimate INTEGER, \
            TotPaid INTEGER, \
            Unpaid INTEGER, \
            UnpaidCons INTEGER, \
            PayReqId INTEGER)",
        [])?;
    let _ = conn.execute("CREATE INDEX MinerHistUserId ON MINER_SS_HIST (UserId)", [])?;
    let _ = conn.execute("CREATE INDEX MinerHistTime ON MINER_SS_HIST (Time)", [])?;

    // Create table PAYREQ Payment Requests
    // ReqAmnt -- Requested amount, Msat
    // PayMethod -- Payment method, "LNAD" Lightning Address  "NOLN" Nostr Lightning
    // PriId -- Primary ID of the reciepient, e.g. LN Address or Nostr ID
    let _ = conn.execute(
        "CREATE TABLE PAYREQ ( \
            Id INTEGER PRIMARY KEY AUTOINCREMENT, \
            MinerId INTEGER, \
            ReqAmnt INTEGER, \
            PayMethod VARCHAR(10), \
            PriId VARCHAR(200), \
            ReqTime INTEGER)",
        [])?;
    let _ = conn.execute("CREATE INDEX PayreqId ON PAYREQ (Id)", [])?;
    let _ = conn.execute("CREATE INDEX PayreqTime ON PAYREQ (ReqTime)", [])?;

    // Create table PAYMENT Payments
    // Status -- 0 NotTried 1 InProgress 2 SuccessFinal 3 FailedRetry 4 FailedFinal
    // ErrorCode -- See error_codes.py
    // PaidAmnt -- The amount paid including fees
    // PaidFee -- The fee, if any (if applicable and known). Fee is included in amount paid.
    let _ = conn.execute(
        "CREATE TABLE PAYMENT ( \
            Id INTEGER PRIMARY KEY AUTOINCREMENT, \
            ReqId INTEGER, \
            CreateTime INTEGER, \
            Status INTEGER, \
            StatusTime INTEGER, \
            ErrorCode INTEGER, \
            ErrorStr VARCHAR(200), \
            RetryCnt INTEGER, \
            FailTime INTEGER, \
            PaidAmnt INTEGER, \
            PaidFee INTEGER, \
            PayTime INTEGER, \
            PayRef VARCHAR(200), \
            SeconId VARCHAR(1000), \
            TertiId VARCHAR(1000), \
            FOREIGN KEY (ReqId) REFERENCES PAYREQ(Id))",
        [])?;
    let _ = conn.execute("CREATE INDEX PaymentId ON PAYMENT (Id)", [])?;
    let _ = conn.execute("CREATE INDEX PaymentReqId ON PAYMENT (ReqId)", [])?;
    let _ = conn.execute("CREATE INDEX PaymentStatusTime ON PAYMENT (StatusTime)", [])?;

    // Note: auto commit

    Ok(())
}

pub fn get_status(conn: &Connection) -> Result<(i32, u32, u32, i32, u32), Box<dyn Error>> {
    let mut stmt = conn.prepare(
        "SELECT \
            LastWorkItemRetrvd, LastBlockRetrvd, LastBlockProcd, LastPaymentProcd, LastWorkItemTimeRetrvd \
            FROM STATUS LIMIT 1")?;

    let res = stmt.query_one([], |row| {
        Ok((
            row.get::<_, i32>(0)?,
            row.get::<_, u32>(1)?,
            row.get::<_, u32>(2)?,
            row.get::<_, i32>(3)?,
            row.get::<_, u32>(4)?,
        ))
    })?;
    Ok(res)
}

fn block_from_row(row: &Row) -> Result<Block, rusqlite::Error> {
    // println!("block_from_row {0:?}", row);
    let b = Block::new(
        row.get::<_, u32>(0)?,
        row.get::<_, String>(1)?,
        row.get::<_, u64>(2)?,
        row.get::<_, u32>(3)?,
        row.get::<_, u64>(4)?,
    );
    // println!("block_from_row {0}", b.block_hash);
    Ok(b)
}

// Doesn't commit
pub fn set_status_last_workitem_retrvd(conntx: &Transaction, newval: i32, new_time_val: u32) -> Result<(), Box<dyn Error>> {
    let _ = conntx.execute(
        "UPDATE STATUS SET LastWorkItemRetrvd = ?1, LastWorkItemTimeRetrvd = ?2",
        [newval, new_time_val as i32])?;
    Ok(())
}

// Doesn't commit
pub fn set_status_last_block_retrvd(conntx: &Transaction, newval: u32) -> Result<(), Box<dyn Error>> {
    let _ = conntx.execute(
        "UPDATE STATUS SET LastBlockRetrvd = ?1",
        (newval,))?;
    Ok(())
}

/*
# Doesn't commit
def set_status_last_block_procd(cursor: sqlite3.Cursor, newval: int):
    cursor.execute("""
        UPDATE STATUS
        SET LastBlockProcd = ?
    """, (newval,))


# Doesn't commit
def set_status_last_payment_procd(cursor: sqlite3.Cursor, newval: int):
    cursor.execute("""
        UPDATE STATUS
        SET LastPaymentProcd = ?
    """, (newval,))


# Get Id of a username string, return Id, or -1 if not found
def userlookup_get_id(cursor: sqlite3.Connection, username_string: str) -> int:
    cursor.execute("SELECT Id FROM USERLOOKUP WHERE String = ?", (username_string,))
    rows = cursor.fetchall()
    if len(rows) >= 1:
        if len(rows[0]) >= 1:
            # Found in DB
            return rows[0][0]
    return -1
*/

// Get Id or insert username string, return Id
// Note: it does'n commit
fn userlookup_get_or_insert_id_nocommit(conn: &Transaction, username_string: &str, typ: u8, time_add: u32) -> Result<u32, Box<dyn Error>> {
    let mut stmt = conn.prepare("SELECT Id FROM USERLOOKUP WHERE String = ?1")?;
    if let Ok(id) = stmt.query_one([username_string], |row| row.get::<_, u32>(0)) {
        // Found in DB
        return Ok(id);
    }

    // Not found, insert
    let mut stmt2 = conn.prepare(
        "INSERT INTO USERLOOKUP \
            (String, Type, TimeAdd) \
            VALUES (?1, ?2, ?3) \
            RETURNING Id \
        ")?;
    if let Ok(id) = stmt2.query_one((username_string, typ, time_add), |row| row.get::<_, u32>(0)) {
        // Added to DB
        return Ok(id);
    }

    Err(format!("ERROR Could not insert original user {username_string} {typ}").into())
}

/*
def userlookup_get_string(cursor: sqlite3.Cursor, id: int) -> str:
    cursor.execute("SELECT String FROM USERLOOKUP WHERE Id = ?", (id,))
    rows = cursor.fetchall()
    s = "?"
    if len(rows) >= 1:
        if len(rows[0]) >= 1:
            s = rows[0][0]
    return s
*/

// Note: It doesn't commit
pub fn insert_work_struct_nocommit(conn: &Transaction, mut w: Work) -> Result<usize, Box<dyn Error>> {
    if w.uname_o_id == 0 {
        w.uname_o_id = userlookup_get_or_insert_id_nocommit(conn, &w.uname_o, 11, w.time_add as u32)?;
    }
    if w.uname_o_wrkr_id == 0 {
        w.uname_o_wrkr_id = userlookup_get_or_insert_id_nocommit(conn, &w.uname_o_wrkr, 12, w.time_add as u32)?;
    }
    if w.uname_u_id == 0 {
        w.uname_u_id = userlookup_get_or_insert_id_nocommit(conn, &w.uname_u, 21, w.time_add as u32)?;
    }
    if w.uname_u_wrkr_id == 0 {
        w.uname_u_wrkr_id = userlookup_get_or_insert_id_nocommit(conn, &w.uname_u_wrkr, 22, w.time_add as u32)?;
    }
    // println!("user ids: {} {} {} {}",
    //     w.uname_o_id, w.uname_o_wrkr_id, w.uname_u_id, w.uname_u_wrkr_id);

    // user_orig_id2 = get_or_insert_orig_user(conn, w.uname_o, w.uname_o_wrkr, w.uname_u_wrkr, w.time_add)
    // # print(f"user_orig_id = {user_orig_id}")
    // user_us_id2 = get_or_insert_us_user(conn, w.uname_u, w.time_add)
    // # print(f"user_us_id = {user_us_id}")

    let cnt = conn.execute(
        "INSERT INTO WORK \
                ( \
                    UNameO, UNameOWrkr, UNameU, UNameUWrkr, \
                    TDiff, TimeAdd, \
                    Payed, PayedTime, PayedRef, \
                    Committed, CommitBlocks, CommitFirstTime, CommitNextTime, Estimate \
                ) \
            VALUES \
                ( \
                    ?1, ?2, ?3, ?4, \
                    ?5, ?6, \
                    ?7, ?8, ?9, \
                    ?10, ?11, ?12, ?13, ?14 \
                )",
        (
            w.uname_o_id, w.uname_o_wrkr_id, w.uname_u_id, w.uname_u_wrkr_id,
            w.tdiff, w.time_add,
            w.payed, w.payed_time, w.payed_ref,
            w.committed, w.commit_blocks, w.commit_first_time, w.commit_next_time, w.estimate
        )
    )?;
    Ok(cnt)
}

/*
# May throw if cannot connect
def insert_work_struct(conn: sqlite3.Connection, w: Work):
    insert_work_struct_nocommit(conn, w)
    conn.commit()


def get_work_count(cursor: sqlite3.Cursor) -> int:
    cursor.execute("SELECT COUNT(*) FROM WORK")
    rows = cursor.fetchall()
    cnt = 0
    if len(rows) >= 1:
        if len(rows[0]) >= 1:
            if rows[0][0] != None:
                cnt = rows[0][0]
    # print(cnt)
    return cnt
*/

pub fn work_get_total_committed(conn: &Connection) -> Result<u64, Box<dyn Error>> {
    let mut stmt = conn.prepare("SELECT SUM(Committed) FROM WORK")?;
    let sum = stmt.query_one((), |row| {
        row.get::<_, u64>(0)
    })?;
    // println!("work_get_total_committed {sum}");
    Ok(sum)
}

pub fn work_get_total_estimated(conn: &Connection) -> Result<u64, Box<dyn Error>> {
    let mut stmt = conn.prepare("SELECT SUM(Estimate) FROM WORK")?;
    let sum = stmt.query_one((), |row| {
        row.get::<_, u64>(0)
    })?;
    // println!("{sum}")
    Ok(sum)
}

/*
def work_get_user_total_committed(cursor: sqlite3.Cursor, user_o_id: int) -> int:
    cursor.execute("SELECT SUM(Committed) FROM WORK WHERE UNameO == ?", (user_o_id,))
    rows = cursor.fetchall()
    sum = 0
    if len(rows) >= 1:
        if len(rows[0]) >= 1:
            if rows[0][0] != None:
                sum = rows[0][0]
    # print(sum)
    return sum


def work_get_user_total_estimated(cursor: sqlite3.Cursor, user_o_id: int) -> int:
    cursor.execute("SELECT SUM(Estimate) FROM WORK WHERE UNameO == ?", (user_o_id,))
    rows = cursor.fetchall()
    sum = 0
    if len(rows) >= 1:
        if len(rows[0]) >= 1:
            if rows[0][0] != None:
                sum = rows[0][0]
    # print(sum)
    return sum


def work_update_nocommit(cursor: sqlite3.Cursor, w: Work):
    cursor.execute("""
        UPDATE WORK
        SET 
            Committed = ?,
            CommitBlocks = ?,
            CommitFirstTime = ?,
            CommitNextTime = ?,
            Estimate = ?
        WHERE Id = ?
    """, (w.committed, w.commit_blocks, w.commit_first_time, w.commit_next_time, w.estimate, w.db_id))
    cursor.fetchall()


# Private
def work_query_custom(cursor: sqlite3.Cursor, condition_string: str, arguments) -> list[Work]:
    cursor.execute("""
        SELECT
            Id, UNameO, UNameOWrkr, UNameU, UNameUWrkr,
            TDiff, TimeAdd,
            Payed, PayedTime, PayedRef,
            Committed, CommitBlocks, CommitFirstTime, CommitNextTime, Estimate
        FROM WORK
        """ + condition_string,
        arguments)
    res = []
    rows = cursor.fetchall()
    for r in rows:
        if len(r) >= 15:
            res.append(Work(
                r[0],
                "?", "?", "?", "?",
                r[1], r[2], r[3], r[4],
                r[5], r[6], r[7], r[8], r[9], r[10], r[11], r[12], r[13], r[14]
            ))
    # print(len(res))
    return res


# Return work items that are to be affected by a new block earning
# Note: usernames are not filled (to save on joins)
def work_get_affected_by_new_block(cursor: sqlite3.Cursor, block_time: int) -> list[Work]:
    return work_query_custom(cursor, """
        WHERE
            CommitBlocks < ? AND
            TimeAdd <= ? AND
            CommitNextTime < ?
        ORDER BY Id ASC
    """, (BLOCKS_WINDOW, block_time, block_time,))


# Return all work items. Can be slow!
def work_get_all(cursor: sqlite3.Cursor, start_time: int) -> list[Work]:
    return work_query_custom(cursor, """
        WHERE TimeAdd >= ?
    """, (start_time,))


# Get work records whose estimate can be updated,
# that is, they are not completely accounted for yet
def work_get_for_estimate_update(cursor: sqlite3.Cursor, birth_time: int) -> list[Work]:
    return work_query_custom(cursor, """
        WHERE
            CommitBlocks < ? AND
            TimeAdd > ?
    """, (BLOCKS_WINDOW, birth_time,))


def work_get_recent(cursor: sqlite3.Cursor) -> Work:
    list = work_query_custom(cursor, """
        ORDER BY TimeAdd DESC
        LIMIT 1
    """, ())
    if len(list) >= 1:
        return list[0]
    return None


def work_get_user_recent(cursor: sqlite3.Cursor, user_id: str) -> Work:
    list = work_query_custom(cursor, """
        WHERE UNameO == ?
        ORDER BY TimeAdd DESC
        LIMIT 1
    """, (user_id,))
    if len(list) >= 1:
        return list[0]
    return None


# Return work count for a period
def work_count_period(cursor: sqlite3.Cursor, start_time: int, end_time: int) -> Work:
    cursor.execute("""
        SELECT COUNT(*)
        FROM WORK
        WHERE TimeAdd >= ? AND TimeAdd < ?
    """, (start_time, end_time,))
    rows = cursor.fetchall()
    cnt = 0
    if len(rows) >= 1:
        if len(rows[0]) >= 1:
            if rows[0][0] != None:
                cnt = rows[0][0]
    return cnt


# Return work count for a period
def work_count_user_period(cursor: sqlite3.Cursor, user_id: str, start_time: int, end_time: int) -> Work:
    cursor.execute("""
        SELECT COUNT(*)
        FROM WORK
        WHERE UNameO == ? AND TimeAdd >= ? AND TimeAdd < ?
    """, (user_id, start_time, end_time,))
    rows = cursor.fetchall()
    cnt = 0
    if len(rows) >= 1:
        if len(rows[0]) >= 1:
            if rows[0][0] != None:
                cnt = rows[0][0]
    return cnt


# Return total work difficulty for a period
def work_total_diff_period(cursor: sqlite3.Cursor, start_time: int, end_time: int) -> Work:
    cursor.execute("""
        SELECT SUM(TDiff)
        FROM WORK
        WHERE TimeAdd >= ? AND TimeAdd < ?
    """, (start_time, end_time,))
    rows = cursor.fetchall()
    tdiff = 0
    if len(rows) >= 1:
        if len(rows[0]) >= 1:
            if rows[0][0] != None:
                tdiff = rows[0][0]
    return tdiff


# Return total work difficulty for a period for a user
def work_total_diff_user_period(cursor: sqlite3.Cursor, user_id: str, start_time: int, end_time: int) -> Work:
    cursor.execute("""
        SELECT SUM(TDiff)
        FROM WORK
        WHERE UNameO == ? AND TimeAdd >= ? AND TimeAdd < ?
    """, (user_id, start_time, end_time,))
    rows = cursor.fetchall()
    tdiff = 0
    if len(rows) >= 1:
        if len(rows[0]) >= 1:
            if rows[0][0] != None:
                tdiff = rows[0][0]
    return tdiff


# Return user count for a period (distinct)
def work_user_count_period(cursor: sqlite3.Cursor, start_time: int, end_time: int) -> Work:
    cursor.execute("""
        SELECT COUNT(DISTINCT UnameO)
        FROM WORK
        WHERE TimeAdd >= ? AND TimeAdd < ?
    """, (start_time, end_time,))
    rows = cursor.fetchall()
    cnt = 0
    if len(rows) >= 1:
        if len(rows[0]) >= 1:
            if rows[0][0] != None:
                cnt = rows[0][0]
    return cnt


# Return top users for a period
def work_users_in_period(cursor: sqlite3.Cursor, start_time: int, end_time: int) -> list[tuple[str, int]]:
    cursor.execute("""
        SELECT USERLOOKUP.String AS User, SUM(WORK.TDiff) AS Diff
        FROM WORK
        LEFT OUTER JOIN USERLOOKUP
        ON WORK.UNameO == USERLOOKUP.Id
        WHERE WORK.TimeAdd >= ? AND WORK.TimeAdd < ?
        GROUP BY WORK.UNameO
        ORDER BY Diff DESC
        LIMIT 7
    """, (start_time, end_time,))
    res = []
    rows = cursor.fetchall()
    for r in rows:
        if len(r) >= 2:
            res.append([r[0], r[1]])
    return res


# Return device count for a period (distinct)
def work_device_count_period(cursor: sqlite3.Cursor, start_time: int, end_time: int) -> Work:
    cursor.execute("""
        SELECT COUNT(DISTINCT UnameUWrkr)
        FROM WORK
        WHERE TimeAdd >= ? AND TimeAdd < ?
    """, (start_time, end_time,))
    rows = cursor.fetchall()
    cnt = 0
    if len(rows) >= 1:
        if len(rows[0]) >= 1:
            if rows[0][0] != None:
                cnt = rows[0][0]
    return cnt


# Return device count for a user for period (distinct)
def work_device_count_user_period(cursor: sqlite3.Cursor, user_id: str, start_time: int, end_time: int) -> Work:
    cursor.execute("""
        SELECT COUNT(DISTINCT UnameUWrkr)
        FROM WORK
        WHERE UNameO == ? AND TimeAdd >= ? AND TimeAdd < ?
    """, (user_id, start_time, end_time,))
    rows = cursor.fetchall()
    cnt = 0
    if len(rows) >= 1:
        if len(rows[0]) >= 1:
            if rows[0][0] != None:
                cnt = rows[0][0]
    return cnt
*/

// Get the blocks after a certain time, oldest first.
// Old time is typically the time of the already processed last block.
pub fn block_get_new_blocks(conn: &Connection, old_time: u32) -> Result<Vec<Block>, Box<dyn Error>> {
    let mut stmt = conn.prepare(
        "SELECT \
            Time, BlockHash, Earning, PoolFee, AccTotalDiff \
            FROM PC_BLOCK \
            WHERE Time > ?1 \
            ORDER BY Time ASC")?;
    let vector = stmt.query_map((old_time,), |row| block_from_row(row))?
        .filter(|blr| blr.is_ok())
        .map(|blr| blr.unwrap())
        .collect::<Vec<Block>>();
    Ok(vector)
}

pub fn block_get_total_earn(conn: &Connection) -> Result<u64, Box<dyn Error>> {
    let mut stmt = conn.prepare("SELECT SUM(Earning) FROM PC_BLOCK")?;
    let sum = stmt.query_one((), |row| {
        row.get::<_, u64>(0)
    })?;
    Ok(sum)
}

pub fn block_get_total_earned(conn: &Connection) -> Result<u64, Box<dyn Error>> {
    let mut stmt = conn.prepare("SELECT SUM(Earning) FROM PC_BLOCK")?;

    let res = stmt.query_one((), |row| {
        row.get::<_, u64>(0)
    })?;
    Ok(res)
}

// Note: Doesn't commit
pub fn block_insert(conntx: &Transaction, block: &Block, now: u32) -> Result<(), Box<dyn Error>> {
    let _ = conntx.execute(
        "INSERT INTO PC_BLOCK \
            (Time, BlockHash, Earning, PoolFee, TimeAddedFirst, TimeUpdated, AccTotalDiff) \
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        (block.time, &block.block_hash, block.earned_sats, block.pool_fee, now, now, block.acc_total_diff))?;
    // println!("inserted");
    Ok(())
}

pub fn block_update_diff_no_commit(conn: &Connection, block_time: u32, new_acc_total_diff: u64) -> Result<(), Box<dyn Error>> {
    let _ = conn.execute(
        "UPDATE PC_BLOCK SET AccTotalDiff = ?1 WHERE Time = ?2",
        (new_acc_total_diff, block_time))?;
    Ok(())
}


// Return the average earnings for the last N blocks.
// Return a tuple: the sum of earnings and the sum of difficulties
pub fn block_get_last_avg_n(conn: &Connection, last_block_count: u32) -> Result<(u64, u64), Box<dyn Error>> {
    // First find the N most recent blocks
    // Clamp to 3 -- 100
    let count = std::cmp::max(std::cmp::min(last_block_count, 100), 3);
    // let mut stmt QQQQ = cursor.execute(f"SELECT Time FROM PC_BLOCK ORDER BY Time DESC LIMIT {count}")
    // rows = cursor.fetchall()
    // if len(rows) == 0:
    //     return [0, 0]
    // lastrow = rows[len(rows) - 1]
    // if len(lastrow) == 0:
    //     return [0, 0]
    // last_block_time = lastrow[0]

    // cursor.execute("""
    //     SELECT SUM(Earning), SUM(AccTotalDiff)
    //     FROM PC_BLOCK
    //     WHERE Time >= ?
    // """, (last_block_time,))
    // rows = cursor.fetchall()
    // sum_e = 0
    // sum_d = 0
    // if len(rows) >= 1:
    //     row = rows[0]
    //     if len(row) >= 1:
    //         sum_e = row[0]
    //     if len(row) >= 2:
    //         sum_d = row[1]
    // return [sum_e, sum_d]
    Err("TODO".into())
}

/*
def miner_ss_exists(cursor: sqlite3.Cursor, id: int) -> bool:
    cursor.execute("SELECT UserId FROM MINER_SS WHERE UserId = ?", (id,))
    rows = cursor.fetchall()
    if len(rows) == 0:
        return False
    return True


def miner_ss_insert_nocommit(cursor: sqlite3.Cursor, ss: MinerSnapshot):
    # History: simply insert
    cursor.execute("""
        INSERT INTO MINER_SS_HIST
        (UserId, Time, TotCommit, TotEstimate, TotPaid, Unpaid, UnpaidCons, PayReqId)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (ss.user_id, ss.time, ss.tot_commit, ss.tot_estimate, ss.tot_paid, ss.unpaid, ss.unpaid_cons, ss.payreq_id,))

    # Update-or-add.
    cursor.execute("""
        UPDATE MINER_SS
        SET Time = ?, TotCommit = ?, TotEstimate = ?, TotPaid = ?, Unpaid = ?, UnpaidCons = ?, PayReqId = ?
        WHERE UserId = ?
        """,
        (ss.time, ss.tot_commit, ss.tot_estimate, ss.tot_paid, ss.unpaid, ss.unpaid_cons, ss.payreq_id, ss.user_id,))

    cursor.execute("SELECT UserId FROM MINER_SS WHERE UserId = ?", (ss.user_id,))
    rows = cursor.fetchall()
    if len(rows) == 0:
        # Not present
        cursor.execute("""
            INSERT INTO MINER_SS
            (UserId, UserS, Time, TotCommit, TotEstimate, TotPaid, Unpaid, UnpaidCons, PayReqId)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
            (ss.user_id, ss.user_s, ss.time, ss.tot_commit, ss.tot_estimate, ss.tot_paid, ss.unpaid, ss.unpaid_cons, ss.payreq_id)
        )


def miner_ss_get_all(conn: sqlite3.Connection) -> list[MinerSnapshot]:
    cursor = conn.cursor()
    cursor.execute("""
        SELECT UserId, UserS, Time, TotCommit, TotEstimate, TotPaid, Unpaid, UnpaidCons, PayReqId 
        FROM MINER_SS
        ORDER BY UserId ASC
    """)
    rows = cursor.fetchall()
    res = []
    for r in rows:
        if len(r) >= 9:
            ss = MinerSnapshot(r[0], r[1], r[2], r[3], r[4], r[5], r[6], r[7], r[8])
            res.append(ss)
    cursor.close()
    return res


# Return Id
def payreq_insert_nocommit(cursor: sqlite3.Cursor, pr: PayRequest) -> int:
    cursor.execute("""
        INSERT INTO PAYREQ
        (MinerId, ReqAmnt, PayMethod, PriId, ReqTime)
        VALUES (?, ?, ?, ?, ?)
        RETURNING Id
        """,
        (pr.miner_id, pr.req_amnt, pr.pay_method, pr.pri_id, pr.req_time))
    rows = cursor.fetchall()
    if len(rows) >= 1:
        row = rows[0]
        if len(row) >= 1:
            return row[0]
    raise Exception(f"ERROR Could not insert pay request {pr.miner_id} {pr.pri_id} {pr.req_amnt}")


def payreq_get_all(conn: sqlite3.Connection) -> list[PayRequest]:
    cursor = conn.cursor()
    cursor.execute("""
        SELECT Id, MinerId, ReqAmnt, PayMethod, PriId, ReqTime 
        FROM PAYREQ
        ORDER BY ReqTime ASC
    """)
    rows = cursor.fetchall()
    res = []
    for r in rows:
        if len(r) >= 6:
            pr = PayRequest(r[0], r[1], r[2], r[3], r[4], r[5])
            res.append(pr)
    cursor.close()
    return res


def payreq_get_id(conn: sqlite3.Connection, id: int) -> PayRequest:
    cursor = conn.cursor()
    cursor.execute("""
        SELECT Id, MinerId, ReqAmnt, PayMethod, PriId, ReqTime 
        FROM PAYREQ
        WHERE Id = ?
        LIMIT 1
    """, (id,))
    rows = cursor.fetchall()
    if len(rows) < 1:
        return None
    r = rows[0]
    if len(r) < 6:
        return None
    pr = PayRequest(r[0], r[1], r[2], r[3], r[4], r[5])
    cursor.close()
    return pr


# Get all payrequests that are non-final (open): all except those for which a Payment
# with final state (2 SuccessFinal or 4 FailedFinal) exists.
def payreq_get_all_non_final(conn: sqlite3.Connection) -> list[tuple[PayRequest, Payment]]:
    cursor = conn.cursor()
    cursor.execute("""
        SELECT
        PAYREQ.Id, PAYREQ.MinerId, PAYREQ.ReqAmnt, PAYREQ.PayMethod, PAYREQ.PriId, PAYREQ.ReqTime,
        PAYMENT.Id, PAYMENT.ReqId, PAYMENT.CreateTime, PAYMENT.Status, PAYMENT.StatusTime, PAYMENT.ErrorCode, PAYMENT.ErrorStr, PAYMENT.RetryCnt, PAYMENT.FailTime, PAYMENT.SeconId, PAYMENT.TertiId, PAYMENT.PaidAmnt, PAYMENT.PaidFee, PAYMENT.PayTime, PAYMENT.PayRef
        FROM PAYREQ
        LEFT OUTER JOIN PAYMENT ON PAYREQ.Id = PAYMENT.ReqId
        WHERE (PAYMENT.Status IS NULL OR (PAYMENT.Status != 2 AND PAYMENT.Status != 4))
        ORDER BY PAYREQ.ReqTime ASC
    """)
    rows = cursor.fetchall()
    res = []
    for r in rows:
        if len(r) >= 21:
            pr = PayRequest(r[0], r[1], r[2], r[3], r[4], r[5])
            paym = Payment(r[6], r[7], r[8], r[9], r[10], r[11], r[12], r[13], r[14], r[15], r[16], r[17], r[18], r[19], r[20])
            res.append([pr, paym])
    cursor.close()
    return res


# Return Id
def payment_insert_nocommit(cursor: sqlite3.Cursor, p: Payment):
    cursor.execute("""
        INSERT INTO PAYMENT
        (ReqId, CreateTime, Status, StatusTime, ErrorCode, ErrorStr, RetryCnt, FailTime, SeconId, TertiId, PaidAmnt, PaidFee, PayTime, PayRef)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?. ?, ?, ?, ?)
        RETURNING Id
        """,
        (p.req_id, p.crete_time, p.status, p.status_time, p.error_code, p.error_str, p.retry_cnt, p.fail_time, p.secon_id, p.terti_id, p.paid_amnt, p.paid_fee, p.pay_time, p.pay_ref))
    rows = cursor.fetchall()
    if len(rows) >= 1:
        row = rows[0]
        if len(row) >= 1:
            return row[0]
    raise Exception(f"ERROR Could not insert payment {p.req_id}")


# Return Id
def payment_update_or_insert_nocommit(cursor: sqlite3.Cursor, p: Payment) -> int:
    cursor.execute("""
        UPDATE PAYMENT
        SET ReqId = ?, CreateTime = ?, Status = ?, StatusTime = ?, ErrorCode = ?, ErrorStr = ?, RetryCnt = ?, FailTime = ?, SeconId = ?, TertiId = ?, PaidAmnt = ?, PaidFee = ?, PayTime = ?, PayRef = ?
        WHERE Id = ?
        """,
        (p.req_id, p.create_time, p.status, p.status_time, p.error_code, p.error_str, p.retry_cnt, p.fail_time, p.secon_id, p.terti_id, p.paid_amnt, p.paid_fee, p.pay_time, p.pay_ref, p.id))

    cursor.execute("SELECT Id FROM PAYMENT WHERE Id = ?", (p.id,))
    rows = cursor.fetchall()
    if len(rows) == 0:
        # Not present
        cursor.execute("""
            INSERT INTO PAYMENT
            (ReqId, CreateTime, Status, StatusTime, ErrorCode, ErrorStr, RetryCnt, FailTime, SeconId, TertiId, PaidAmnt, PaidFee, PayTime, PayRef)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING Id
            """,
            (p.req_id, p.create_time, p.status, p.status_time, p.error_code, p.error_str, p.retry_cnt, p.fail_time, p.secon_id, p.terti_id, p.paid_amnt, p.paid_fee, p.pay_time, p.pay_ref))
        rows = cursor.fetchall()
        if len(rows) >= 1:
            if len(rows[0]) >= 1:
                return rows[0][0]
        raise Exception(f"Could not insert into payment, {p.id} {p.req_id}")
    else:
        return p.id


# Get the total paid amount to a miner,
# successful ones and also including request-only, NotTried, InProgress and NonfinalFailure
# (excluding FinalFailure)
# Uses PAYREQ and PAYMENT
def payment_get_total_paid_to_miner(cursor: sqlite3.Cursor, miner_id: int) -> int:
    # # Debug
    # if False:
    #     cursor.execute("""
    #         SELECT PAYREQ.MinerId, PAYREQ.Id, PAYMENT.ReqId, PAYMENT.PaidAmnt, PAYMENT.Status
    #         FROM PAYREQ
    #         LEFT OUTER JOIN PAYMENT
    #         ON PAYREQ.Id = PAYMENT.ReqId
    #         WHERE PAYREQ.MinerId = ?
    #         AND (PAYMENT.Status IS NULL OR PAYMENT.Status != 4)
    #     """, (miner_id,))
    #     rows = cursor.fetchall()
    #     print(f"QQQ {rows}")

    cursor.execute("""
        SELECT SUM(PAYMENT.PaidAmnt)
        FROM PAYREQ
        LEFT OUTER JOIN PAYMENT
        ON PAYREQ.Id = PAYMENT.ReqId
        WHERE PAYREQ.MinerId = ?
        AND (PAYMENT.Status IS NULL OR PAYMENT.Status != 4)
    """, (miner_id,))
    rows = cursor.fetchall()
    # print(f"{rows}")
    sum = 0
    if len(rows) >= 1:
        if len(rows[0]) >= 1:
            if rows[0][0] != None:
                sum = rows[0][0]
    return sum


# def payment_get_latest_update_time(cursor: sqlite3.Cursor) -> int:
#     cursor.execute("""
#         SELECT StatusTime
#         FROM PAYMENT
#         ORDER BY StatusTime DESC
#         LIMIT 1
#     """)
#     rows = cursor.fetchall()
#     print(f"QQQ {rows}")
#     if len(rows) >= 1:
#         if len(rows[0]) >= 1:
#             return rows[0][0]
#     return None


# Get all payments updated after a certain time
# Time comparison is strict
def payment_get_all_after_time(conn: sqlite3.Connection, time: int) -> list[tuple[Payment, PayRequest]]:
    cursor = conn.cursor()
    cursor.execute("""
        SELECT
        PAYREQ.Id, PAYREQ.MinerId, PAYREQ.ReqAmnt, PAYREQ.PayMethod, PAYREQ.PriId, PAYREQ.ReqTime,
        PAYMENT.Id, PAYMENT.ReqId, PAYMENT.CreateTime, PAYMENT.Status, PAYMENT.StatusTime, PAYMENT.ErrorCode, PAYMENT.ErrorStr, PAYMENT.RetryCnt, PAYMENT.FailTime, PAYMENT.SeconId, PAYMENT.TertiId, PAYMENT.PaidAmnt, PAYMENT.PaidFee, PAYMENT.PayTime, PAYMENT.PayRef
        FROM PAYMENT
        INNER JOIN PAYREQ ON PAYMENT.ReqId = PAYREQ.Id
        WHERE PAYMENT.StatusTime > ?
        ORDER BY PAYMENT.StatusTime ASC
    """, (time,))
    rows = cursor.fetchall()
    res = []
    for r in rows:
        if len(r) >= 21:
            pr = PayRequest(r[0], r[1], r[2], r[3], r[4], r[5])
            paym = Payment(r[6], r[7], r[8], r[9], r[10], r[11], r[12], r[13], r[14], r[15], r[16], r[17], r[18], r[19], r[20])
            res.append([paym, pr])
    cursor.close()
    return res


# Get all payments updated after a certain time, for a user
# Time comparison is strict
def payment_get_all_after_time_user(conn: sqlite3.Connection, time: int, user_id: int) -> list[tuple[Payment, PayRequest]]:
    cursor = conn.cursor()
    cursor.execute("""
        SELECT
        PAYREQ.Id, PAYREQ.MinerId, PAYREQ.ReqAmnt, PAYREQ.PayMethod, PAYREQ.PriId, PAYREQ.ReqTime,
        PAYMENT.Id, PAYMENT.ReqId, PAYMENT.CreateTime, PAYMENT.Status, PAYMENT.StatusTime, PAYMENT.ErrorCode, PAYMENT.ErrorStr, PAYMENT.RetryCnt, PAYMENT.FailTime, PAYMENT.SeconId, PAYMENT.TertiId, PAYMENT.PaidAmnt, PAYMENT.PaidFee, PAYMENT.PayTime, PAYMENT.PayRef
        FROM PAYMENT
        INNER JOIN PAYREQ ON PAYMENT.ReqId = PAYREQ.Id
        WHERE PAYMENT.StatusTime > ? AND PAYREQ.MinerId == ?
        ORDER BY PAYMENT.StatusTime ASC
    """, (time, user_id))
    rows = cursor.fetchall()
    res = []
    for r in rows:
        if len(r) >= 21:
            pr = PayRequest(r[0], r[1], r[2], r[3], r[4], r[5])
            paym = Payment(r[6], r[7], r[8], r[9], r[10], r[11], r[12], r[13], r[14], r[15], r[16], r[17], r[18], r[19], r[20])
            res.append([paym, pr])
    cursor.close()
    return res


# Get all-time payments sum. Only successful payments are included
def payment_get_total_amount(conn: sqlite3.Connection) -> tuple[int, int] | None:
    cursor = conn.cursor()
    cursor.execute("""
        SELECT SUM(PAYMENT.PaidAmnt), SUM(PAYMENT.PaidFee)
        FROM PAYMENT
        WHERE PAYMENT.Status == 2
    """)
    rows = cursor.fetchall()
    # print(rows)
    if len(rows) >= 1:
        r = rows[0]
        if len(r) >= 2:
            return (int(r[0]), int(r[1]))
    return None


# Get all-time payments sum for user. Only successful payments are included
def payment_get_total_amount_for_user(conn: sqlite3.Connection, user_id: int) -> tuple[int, int] | None:
    cursor = conn.cursor()
    cursor.execute("""
        SELECT SUM(PAYMENT.PaidAmnt), SUM(PAYMENT.PaidFee)
        FROM PAYMENT
        INNER JOIN PAYREQ ON PAYMENT.ReqId = PAYREQ.Id
        WHERE PAYMENT.Status == 2
        AND PAYREQ.MinerId == ?
    """, (user_id,))
    rows = cursor.fetchall()
    # print(rows)
    if len(rows) >= 1:
        r = rows[0]
        if len(r) >= 2:
            return (int(r[0]), int(r[1]))
    return None
 */

 #[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn create_test_db(conn: &Connection) -> Result<(), Box<dyn Error>> {
        // Create an empty database
        db_update_0_3(conn)?;
        Ok(())
    }

    #[test]
    fn test_status() -> Result<(), Box<dyn Error>> {
        let mut conn = Connection::open_in_memory()?;
        create_test_db(&conn)?;

        { // get values
            let (
                last_workitem_retrvd,
                last_block_retrvd,
                last_block_procd,
                last_payment_procd,
                last_workitem_time_retrvd
            ) = get_status(&conn).unwrap();
            assert_eq!(last_workitem_retrvd, -1);
            assert_eq!(last_workitem_time_retrvd, 0);
            assert_eq!(last_block_retrvd, 0);
            assert_eq!(last_block_procd, 0);
            assert_eq!(last_payment_procd, 0);
        }
        let tx = conn.transaction().unwrap();
        let _ = set_status_last_workitem_retrvd(&tx, 4, 3000).unwrap();
        { // get values
            let (last_workitem_retrvd, _, _, _, last_workitem_time_retrvd) = get_status(&tx).unwrap();
            assert_eq!(last_workitem_retrvd, 4);
            assert_eq!(last_workitem_time_retrvd, 3000);
        }

        Ok(())
    }
}
