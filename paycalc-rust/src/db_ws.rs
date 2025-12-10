use crate::dto_ws::Work;

use rusqlite::{Connection, Row};
use std::vec::Vec;
use std::error::Error;

fn work_from_row(row: &Row) -> Result<Work, rusqlite::Error> {
    // println!("work_From_row {0:?}", row);
    let w = Work::new(
        row.get(0)?,
        &row.get::<_, String>(1)?,
        &row.get::<_, String>(2)?,
        &row.get::<_, String>(3)?,
        &row.get::<_, String>(4)?,
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
pub fn get_work_after_id(conn: &Connection, start_id: u32, start_time: u32, limit: u32) -> Result<Vec<Work>, Box<dyn Error>> {
    let query_str =
        "SELECT WORK.Id, ORUSER.UNameO, ORUSER.UNameO_wrkr, USUSER.UNameU, ORUSER.UNameU_wrkr, WORK.TDiff, WORK.TimeAdd, WORK.TimeCalc, WORK.CalcPayout \
        FROM WORK \
        LEFT OUTER JOIN ORUSER \
        ON WORK.UNameO = ORUSER.Id \
        LEFT OUTER JOIN USUSER \
        ON WORK.UNameU = USUSER.Id \
        WHERE WORK.Id > ?1 AND WORK.TimeAdd >= ?2 \
        ORDER BY WORK.Id ASC ";

    let vector = if limit == 0 {
        let params = [
            start_id.to_string(),
            start_time.to_string(),
        ];
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
    let mut stmt = conn.prepare(
        "SELECT COUNT(*) FROM WORK"
    )?;

    let res = stmt.query_one([], |row| {
        row.get::<_, u32>(0)
    })?;
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
            []
        )?;
        let _ = conn.execute("CREATE INDEX WorkTimeAdd ON WORK (TimeAdd);", [])?;
        Ok(())
    }

    fn create_test_db(conn: &Connection) -> Result<(), Box<dyn Error>> {
        // Create a test database with WORK table
        db_setup_1(&conn)?;
        conn.execute("INSERT INTO ORUSER (Id, UNameO, UNameO_wrkr, UNameU_wrkr, TimeAdd) VALUES (11, 'uname_o_11', 'wrk11', 'uname_u_11', 100);", [])?;
        conn.execute("INSERT INTO USUSER (Id, UNameU, TimeAdd) VALUES (12, 'uname_u_12', 100);", [])?;
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
    fn test_get_work_count() -> Result<(), Box<dyn Error>> {
        let connection = Connection::open_in_memory()?;
        create_test_db(&connection)?;

        // Test our function
        let count = get_work_count(&connection)?;
        assert_eq!(count, 5);

        // delete_test_db()?;

        Ok(())
    }

    #[test]
    fn test_get_work_after_id() -> Result<(), Box<dyn Error>> {
        let connection = Connection::open_in_memory()?;
        create_test_db(&connection)?;

        { // all
            let count = get_work_after_id(&connection, 0, 1, 0)?;
            assert_eq!(count.iter().len(), 5);
        }
        { // later ID
            let count = get_work_after_id(&connection, 2, 1, 0)?;
            assert_eq!(count.iter().len(), 3);
        }
        { // later time
            let count = get_work_after_id(&connection, 0, 1_002_500, 0)?;
            assert_eq!(count.iter().len(), 2);
        }
        { // limit
            let count = get_work_after_id(&connection, 0, 1, 2)?;
            assert_eq!(count.iter().len(), 2);
        }

        // delete_test_db()?;

        Ok(())
    }
}
