use crate::dto_ws::Work;
use crate::username::split_full_username;

use rusqlite::{Connection, Row};
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn db_setup_1(conn: &Connection) -> Result<(), Box<dyn Error>> {
    // Create table ORUser  (ORiginal User)
    conn.execute(
        "CREATE TABLE ORUSER \
            (Id INTEGER PRIMARY KEY AUTOINCREMENT, UNameO VARCHAR(100), UNameO_wrkr VARCHAR(100), UNameU_wrkr VARCHAR(100), TimeAdd INTEGER);",
        [],
    )?;

    // Create table USUser  (UpStream User)
    conn.execute(
        "CREATE TABLE USUSER \
            (Id INTEGER PRIMARY KEY AUTOINCREMENT, UNameU VARCHAR(100), TimeAdd INTEGER);",
        [],
    )?;

    // Create table WORK
    // UNameO -- original username (without worker name)
    // UNameO_wrkr - Optional worker in the original username
    // UNameU -- upstream username (without worker name)
    // UNameU_wrkr - Worker in the upstream username
    // TDiff - The target difficulty of the work request
    // TimeAdd - Time when package was added
    // TimeCalc - Time when payout was calculated
    // CalcPayout - The calculated estimated payout, sats
    conn.execute(
        "CREATE TABLE WORK (\
            Id INTEGER PRIMARY KEY AUTOINCREMENT,\
            UNameO INTEGER,\
            UNameU INTEGER,\
            TDiff INTEGER,\
            TimeAdd INTEGER,\
            TimeCalc INTEGER,\
            CalcPayout INTEGER,\
            FOREIGN KEY (UNameO) REFERENCES ORUSER(Id)\
            FOREIGN KEY (UNameU) REFERENCES USUSER(Id)\
        );",
        [],
    )?;
    conn.execute("CREATE INDEX WorkTimeAdd ON WORK (TimeAdd);", [])?;

    Ok(())
}

// Get or insert original username, returns the row Id
fn get_or_insert_orig_user(
    conn: &Connection,
    uname_o: &str,
    uname_o_wrkr: &str,
    uname_u_wrkr: &str,
    time_add: f64,
) -> Result<i64, Box<dyn Error>> {
    let existing: Result<i64, _> = conn.query_row(
        "SELECT Id FROM ORUSER WHERE UNameO = ?1 AND UNameO_wrkr = ?2",
        [uname_o, uname_o_wrkr],
        |row| row.get(0),
    );
    if let Ok(id) = existing {
        return Ok(id);
    }

    conn.execute(
        "INSERT INTO ORUSER (UNameO, UNameO_wrkr, UNameU_wrkr, TimeAdd) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![uname_o, uname_o_wrkr, uname_u_wrkr, time_add],
    )?;

    let id: i64 = conn.query_row(
        "SELECT Id FROM ORUSER WHERE UNameO = ?1 AND UNameO_wrkr = ?2",
        [uname_o, uname_o_wrkr],
        |row| row.get(0),
    )?;
    Ok(id)
}

// Get or insert upstream username, returns the row Id
fn get_or_insert_us_user(
    conn: &Connection,
    uname_u: &str,
    time_add: f64,
) -> Result<i64, Box<dyn Error>> {
    let existing: Result<i64, _> = conn.query_row(
        "SELECT Id FROM USUSER WHERE UNameU = ?1",
        [uname_u],
        |row| row.get(0),
    );
    if let Ok(id) = existing {
        return Ok(id);
    }

    conn.execute(
        "INSERT INTO USUSER (UNameU, TimeAdd) VALUES (?1, ?2)",
        rusqlite::params![uname_u, time_add],
    )?;

    let id: i64 = conn.query_row(
        "SELECT Id FROM USUSER WHERE UNameU = ?1",
        [uname_u],
        |row| row.get(0),
    )?;
    Ok(id)
}

pub fn insert_work_raw(conn: &Connection, w: Work) -> Result<(), Box<dyn Error>> {
    let user_orig_id = get_or_insert_orig_user(
        conn,
        &w.uname_o,
        &w.uname_o_wrkr,
        &w.uname_u_wrkr,
        w.time_add,
    )?;
    let user_us_id = get_or_insert_us_user(conn, &w.uname_u, w.time_add)?;

    conn.execute(
        "INSERT INTO WORK (UNameO, UNameU, TDiff, TimeAdd, TimeCalc, CalcPayout) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![user_orig_id, user_us_id, w.tdiff, w.time_add, w.time_calc, w.calc_payout],
    )?;

    Ok(())
}

pub fn insert_work_fullname(
    conn: &Connection,
    uname_o: &str,
    uname_u: &str,
    tdiff: u32,
) -> Result<(), Box<dyn Error>> {
    let (uname_o, uname_o_wrkr) = split_full_username(uname_o);
    let (uname_u, uname_u_wrkr) = split_full_username(uname_u);
    let time_add = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
    let w = Work {
        db_id: 0,
        uname_o,
        uname_o_wrkr,
        uname_u,
        uname_u_wrkr,
        tdiff,
        time_add,
        time_calc: 0,
        calc_payout: 0,
    };

    insert_work_raw(conn, w)
}

fn work_from_row(row: &Row) -> Result<Work, rusqlite::Error> {
    // println!("work_From_row {0:?}", row);
    let w = Work::new(
        row.get(0)?,
        row.get::<_, String>(1)?,
        row.get::<_, String>(2)?,
        row.get::<_, String>(3)?,
        row.get::<_, String>(4)?,
        row.get(5)?,
        row.get(6)?,
        row.get(7)?,
        row.get(8)?,
    );
    // println!("work_From_row {0}", w.db_id);
    Ok(w)
}

/// - start_id: Start after this ID,exclusive
/// - start_time: Start at this time, inclusive
/// - limit: limit the number of entries returned (0=unlimited)
pub fn get_work_after_id(
    conn: &Connection,
    start_id: i32,
    start_time: u32,
    limit: u32,
) -> Result<Vec<Work>, Box<dyn Error>> {
    let query_str = "SELECT WORK.Id, ORUSER.UNameO, ORUSER.UNameO_wrkr, USUSER.UNameU, ORUSER.UNameU_wrkr, WORK.TDiff, WORK.TimeAdd, WORK.TimeCalc, WORK.CalcPayout \
        FROM WORK \
        LEFT OUTER JOIN ORUSER \
        ON WORK.UNameO = ORUSER.Id \
        LEFT OUTER JOIN USUSER \
        ON WORK.UNameU = USUSER.Id \
        WHERE WORK.Id > ?1 AND WORK.TimeAdd >= ?2 \
        ORDER BY WORK.Id ASC ";

    let vector = if limit == 0 {
        let params = [start_id.to_string(), start_time.to_string()];
        let mut stmt = conn.prepare(query_str)?;
        stmt.query_map(params, |row| work_from_row(row))?
            .filter(|wir| wir.is_ok())
            .map(|wir| wir.unwrap())
            .collect::<Vec<Work>>()
    } else {
        let query_with_limit = query_str.to_string() + " LIMIT ?3 ;";
        let params = [
            start_id.to_string(),
            start_time.to_string(),
            limit.to_string(),
        ];
        let mut stmt = conn.prepare(&query_with_limit)?;
        stmt.query_map(params, |row| work_from_row(row))?
            .filter(|wir| wir.is_ok())
            .map(|wir| wir.unwrap())
            .collect::<Vec<Work>>()
    };
    Ok(vector)
}

pub fn get_work_count(conn: &Connection) -> Result<u32, Box<dyn Error>> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM WORK")?;

    let res = stmt.query_one([], |row| Ok(row.get::<_, u32>(0).unwrap_or(0)))?;
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn db_setup_1(conn: &Connection) -> Result<(), Box<dyn Error>> {
        // Create table ORUser  (ORiginal User)
        let _ = conn.execute(
            "CREATE TABLE ORUSER \
                (Id INTEGER PRIMARY KEY AUTOINCREMENT, UNameO VARCHAR(100), UNameO_wrkr VARCHAR(100), UNameU_wrkr VARCHAR(100), TimeAdd INTEGER);",
            [],
        )?;

        // Create table USUser  (UpStream User)
        let _ = conn.execute(
            "CREATE TABLE USUSER \
                (Id INTEGER PRIMARY KEY AUTOINCREMENT, UNameU VARCHAR(100), TimeAdd INTEGER);",
            [],
        )?;

        let _ = conn.execute(
            "CREATE TABLE WORK (\
                Id INTEGER PRIMARY KEY AUTOINCREMENT,\
                UNameO INTEGER,\
                UNameU INTEGER,\
                TDiff INTEGER,\
                TimeAdd INTEGER,\
                TimeCalc INTEGER,\
                CalcPayout INTEGER, \
                FOREIGN KEY (UNameO) REFERENCES ORUSER(Id) \
                FOREIGN KEY (UNameU) REFERENCES USUSER(Id) \
            );",
            [],
        )?;
        let _ = conn.execute("CREATE INDEX WorkTimeAdd ON WORK (TimeAdd);", [])?;
        Ok(())
    }

    fn create_test_db(conn: &Connection) -> Result<(), Box<dyn Error>> {
        // Create a test database with WORK table
        db_setup_1(&conn)?;
        conn.execute("INSERT INTO ORUSER (Id, UNameO, UNameO_wrkr, UNameU_wrkr, TimeAdd) VALUES (11, 'uname_o_11', 'wrk11', 'uname_u_11', 100);", [])?;
        conn.execute(
            "INSERT INTO USUSER (Id, UNameU, TimeAdd) VALUES (12, 'uname_u_12', 100);",
            [],
        )?;
        for i in 0..5 {
            let time_add = 1_000_000 + i * 1_000;
            conn.execute("INSERT INTO WORK (UNameO, UNameU, TDiff, TimeAdd, TimeCalc, CalcPayout) VALUES (11, 12, 131072, ?1, 0, 0);", [time_add.to_string()])?;
        }
        Ok(())
    }

    // fn delete_test_db() -> Result<(), Box<dyn Error>> {
    //     // Clean up any existing test database
    //     let _ = fs::remove_file(TEST_DB_FILENAME);
    //     Ok(())
    // }

    #[test]
    fn test_get_work_count() {
        let conn = Connection::open_in_memory().unwrap();
        create_test_db(&conn).unwrap();

        // Test our function
        let count = get_work_count(&conn).unwrap();
        assert_eq!(count, 5);

        // delete_test_db()?;
    }

    #[test]
    fn test_get_work_after_id() {
        let conn = Connection::open_in_memory().unwrap();
        create_test_db(&conn).unwrap();

        {
            // all
            let count = get_work_after_id(&conn, 0, 1, 0).unwrap();
            assert_eq!(count.iter().len(), 5);
        }
        {
            // later ID
            let count = get_work_after_id(&conn, 2, 1, 0).unwrap();
            assert_eq!(count.iter().len(), 3);
        }
        {
            // later time
            let count = get_work_after_id(&conn, 0, 1_002_500, 0).unwrap();
            assert_eq!(count.iter().len(), 2);
        }
        {
            // limit
            let count = get_work_after_id(&conn, 0, 1, 2).unwrap();
            assert_eq!(count.iter().len(), 2);
        }
    }
}
