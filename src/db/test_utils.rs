// Imports
use chrono::{DateTime, Duration, NaiveTime, TimeZone, Utc};
use rusqlite::Connection;

// Create an in-memory test DB
pub fn setup() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute("PRAGMA foreign_keys = ON;", []).unwrap();
    conn.execute_batch(include_str!("create_tables.sql"))
        .unwrap();
    conn
}

// Helper function to create a DateTime
pub fn create_date_time(str: &str) -> DateTime<Utc> {
    let time = NaiveTime::parse_from_str(str, "%H:%M").unwrap();
    let date = Utc::now().date_naive() - Duration::days(1);
    Utc.from_utc_datetime(&date.and_time(time))
}
