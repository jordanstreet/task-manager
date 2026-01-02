// Imports
use chrono::{DateTime, Utc};
use rusqlite::{Connection, Statement, params};

// Charge code definition
#[derive(Debug, PartialEq)]
pub struct ChargeCode {
    pub id: i64,
    pub code: String,
    pub details: String,
    pub open: DateTime<Utc>,
    pub close: Option<DateTime<Utc>>,
}

// Execute an SQL query and return a vector of charge codes
fn map_to_charge_codes(stmt: &mut Statement) -> rusqlite::Result<Vec<ChargeCode>> {
    // Execute the query and map rows
    let rows = stmt.query_map([], |row| {
        Ok(ChargeCode {
            id: row.get("id")?,
            code: row.get("code")?,
            details: row.get("details")?,
            open: row.get("open")?,
            close: row.get("close")?,
        })
    })?;

    // Create the charge code vector
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

// Get all charge codes
pub fn get_all_charge_codes(conn: &Connection) -> rusqlite::Result<Vec<ChargeCode>> {
    let mut stmt = conn.prepare("SELECT * FROM charge_codes ORDER BY id")?;
    map_to_charge_codes(&mut stmt)
}

// Get active charge codes
pub fn get_active_charge_codes(conn: &Connection) -> rusqlite::Result<Vec<ChargeCode>> {
    let mut stmt = conn.prepare("SELECT * FROM charge_codes WHERE close IS NULL ORDER BY id")?;
    map_to_charge_codes(&mut stmt)
}

// Get charge code ID
pub fn get_charge_code_id(conn: &Connection, code: &str) -> rusqlite::Result<i64> {
    conn.query_one(
        "SELECT id FROM charge_codes WHERE code = ? AND close IS NULL",
        params![code],
        |row| row.get("id"),
    )
}

// Add charge code
pub fn add_charge_code(conn: &Connection, code: &str, details: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO charge_codes (code, details, open) VALUES (?, ?, ?)",
        params![code, details, Utc::now()],
    )?;
    Ok(())
}

// Close charge code
pub fn close_charge_code(conn: &Connection, code: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE charge_codes SET close = ? WHERE code = ? AND close IS NULL",
        params![Utc::now(), code],
    )?;
    Ok(())
}

// Delete charge code
pub fn delete_charge_code(conn: &Connection, id: i64) -> rusqlite::Result<usize> {
    conn.execute("DELETE FROM charge_codes WHERE id = ?", params![id])
}

// Update charge code details
pub fn update_charge_code_details(
    conn: &Connection,
    id: i64,
    details: &str,
) -> rusqlite::Result<usize> {
    conn.execute(
        "UPDATE charge_codes SET details = ? WHERE id = ?",
        params![details, id],
    )
}
