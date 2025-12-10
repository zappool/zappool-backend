use crate::common::get_db_update_versions_from_args;
use rusqlite::{Connection, Transaction};
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

pub fn get_status(conn: &Connection) -> Result<(i32, u32, u32, u32, u32), Box<dyn Error>> {
    let mut stmt = conn.prepare(
        "SELECT \
            LastWorkItemRetrvd, LastBlockRetrvd, LastBlockProcd, LastPaymentProcd, LastWorkItemTimeRetrvd \
            FROM STATUS LIMIT 1")?;

    let res = stmt.query_one([], |row| {
        Ok((
            row.get::<_, i32>(0)?,
            row.get::<_, u32>(1)?,
            row.get::<_, u32>(2)?,
            row.get::<_, u32>(3)?,
            row.get::<_, u32>(4)?,
        ))
    })?;
    Ok(res)
}

// Doesn't commit
pub fn set_status_last_workitem_retrvd(tx: &Transaction, newval: i32, new_time_val: u32) -> Result<(), Box<dyn Error>> {
    let _ = tx.execute(
        "UPDATE STATUS \
            SET LastWorkItemRetrvd = ?1, LastWorkItemTimeRetrvd = ?2",
        [newval, new_time_val as i32])?;
    Ok(())
}

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
