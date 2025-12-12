use dotenv;
use rusqlite::Connection;
use rusqlite::OpenFlags;
use std::env;
use std::error::Error;
use std::fs;

// Return the data dir: the first arg or "."
pub fn get_data_dir() -> String {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Read DB_DIR from environment variables
    match env::var("DB_DIR") {
        Err(_) => {
            let local_dir = env::current_dir().unwrap();
            println!("Using local directory as data dir, ({})", local_dir.to_str().unwrap_or_default());
            local_dir.to_str().unwrap_or_default().to_string()
        }
        Ok(data_dir) => {
            println!("Using data dir from env: '{data_dir}'");
            data_dir
        }
    }
}

// Check and return full path of a DB file
pub fn get_db_file(db_file_name: &str, create_mode: bool) -> String {
    let data_dir = get_data_dir();
    let db_file_name = if create_mode {
        "_new_".to_string() + db_file_name
    } else {
        db_file_name.to_string()
    };
    let dbfile = data_dir + "/" + &db_file_name;
    if !create_mode {
        if !fs::exists(&dbfile).unwrap_or(false) {
            println!("DB file does not exist! {dbfile}");
            std::process::exit(-1);
        }
    }
    println!("Using data file: '{dbfile}'");
    dbfile
}

pub fn get_db_update_versions_from_args(default_to: u8) -> (u8, u8) {
    let mut vto = default_to;
    let mut vfrom = vto - 1;

    let args: Vec<String> = env::args().collect();
    if args.len() >= 3 {
        if let Ok(x) = args[1].parse::<u8>() {
            vfrom = x;
        }
        if let Ok(x) = args[2].parse::<u8>() {
            vto = x;
        }
    }

    println!("DB update versions: v{vfrom} --> v{vto}");
    return (vto, vfrom)
}

fn get_current_db_version(dbfile: &str) -> Result<u8, Box<dyn Error>> {
    let conn = Connection::open_with_flags(dbfile, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let mut stmt = conn.prepare("SELECT Version FROM VERSION LIMIT 1")?;
    let version = stmt.query_one([], |row| {
        row.get::<_, u8>(0)
    })?;
    Ok(version)
}

pub fn print_current_db_version(dbfile: &str) {
    if let Ok(ver) = get_current_db_version(dbfile) {
        println!("Current DB version: v{ver}  ({dbfile})");
    }
}
