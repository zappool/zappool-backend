use crate::dto_oc::BlockEarning;

use rusqlite::{Connection, Row};
use std::vec::Vec;
use std::error::Error;

fn blockearning_from_row(row: &Row) -> Result<BlockEarning, rusqlite::Error> {
    // println!("blockearning_from_row {0:?}", row);
    let w = BlockEarning::new(
        row.get(0)?,
        row.get::<_, String>(1)?,
        row.get(2)?,
        row.get(3)?,
    );
    // println!("blockearning_from_row {0}", w.block_hash);
    Ok(w)
}

pub fn get_new_blocks(conn: &Connection, old_time: u32) -> Result<Vec<BlockEarning>, Box<dyn Error>> {
    let query_str =
        "SELECT Time, BlockHash, Earning, PoolFee, TimeAddedFirst, TimeUpdated \
        FROM OC_BLOCK_EARN \
        WHERE Time > ?1 \
        ORDER BY Time ASC ";

    let mut stmt = conn.prepare(query_str)?;
    let vector = stmt.query_map([old_time], |row| blockearning_from_row(row))?
        .filter(|ber| ber.is_ok())
        .map(|ber| ber.unwrap())
        .collect::<Vec<BlockEarning>>();
    Ok(vector)
}

pub fn count_new_blocks(conn: &Connection, old_time: u32) -> Result<u32, Box<dyn Error>> {
    let mut stmt = conn.prepare(
        "SELECT COUNT(*) FROM OC_BLOCK_EARN WHERE Time > ?"
    )?;

    let res = stmt.query_one([old_time], |row| {
        Ok(row.get::<_, u32>(0))
    })??;
    Ok(res)
}


#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn db_setup_1(conn: &Connection) -> Result<(), Box<dyn Error>> {
        // Create table OC_BLOCK_EARN
        let _ = conn.execute(
            "CREATE TABLE OC_BLOCK_EARN \
            (Time INTEGER, BlockHash VARCHAR(100), Earning INTEGER, PoolFee INTEGER, TimeAddedFirst INTEGER, TimeUpdated INTEGER);",
            [],
        )?;
        Ok(())
    }

    fn create_test_db(conn: &Connection) -> Result<(), Box<dyn Error>> {
        // Create a test database with WORK table
        db_setup_1(&conn)?;
        conn.execute("INSERT INTO OC_BLOCK_EARN (Time, BlockHash, Earning, PoolFee, TimeAddedFirst, TimeUpdated) VALUES (1001, 'block_01', 11, 1, 1001, 1001);", [])?;
        conn.execute("INSERT INTO OC_BLOCK_EARN (Time, BlockHash, Earning, PoolFee, TimeAddedFirst, TimeUpdated) VALUES (1101, 'block_02', 22, 2, 1101, 1101);", [])?;
        Ok(())
    }

    #[test]
    fn test_count_new_blocks() -> Result<(), Box<dyn Error>> {
        let connection = Connection::open_in_memory()?;
        create_test_db(&connection)?;

        // Test our function
        let count = count_new_blocks(&connection, 900)?;
        assert_eq!(count, 2);

        Ok(())
    }

    #[test]
    fn test_get_new_blocks() -> Result<(), Box<dyn Error>> {
        let connection = Connection::open_in_memory()?;
        create_test_db(&connection)?;

        { // all
            let count = get_new_blocks(&connection, 900)?;
            assert_eq!(count.iter().len(), 2);
        }
        { // later ID
            let count = get_new_blocks(&connection, 1050)?;
            assert_eq!(count.iter().len(), 1);
        }
        { // ID just below a block
            let count = get_new_blocks(&connection, 1000)?;
            assert_eq!(count.iter().len(), 2);
        }
        { // ID just at a block
            let count = get_new_blocks(&connection, 1001)?;
            assert_eq!(count.iter().len(), 1);
        }
        { // ID just after a block
            let count = get_new_blocks(&connection, 1002)?;
            assert_eq!(count.iter().len(), 1);
        }

        Ok(())
    }
}
