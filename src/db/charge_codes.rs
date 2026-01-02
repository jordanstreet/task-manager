// Imports
use chrono::{DateTime, Utc};
use rusqlite::{Connection, Result, Row, Statement, params};

// Charge code definition
#[derive(Debug, PartialEq)]
pub struct ChargeCode {
    pub id: i64,
    pub code: String,
    pub details: String,
    pub open: DateTime<Utc>,
    pub close: Option<DateTime<Utc>>,
}

// Map a SQL row to a charge code
fn map_row_to_charge_code(row: &Row) -> Result<ChargeCode> {
    Ok(ChargeCode {
        id: row.get("id")?,
        code: row.get("code")?,
        details: row.get("details")?,
        open: row.get("open")?,
        close: row.get("close")?,
    })
}

// Execute an SQL query and return a vector of charge codes
fn map_to_charge_codes(stmt: &mut Statement) -> Result<Vec<ChargeCode>> {
    // Execute the query and map rows
    let rows = stmt.query_map([], map_row_to_charge_code)?;

    // Create the charge code vector
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

// Get charge code by ID
pub fn get_charge_code(conn: &Connection, id: i64) -> Result<ChargeCode> {
    conn.query_one(
        "SELECT * FROM charge_codes WHERE id = ?",
        params![id],
        map_row_to_charge_code,
    )
}

// Get all charge codes
pub fn get_all_charge_codes(conn: &Connection) -> Result<Vec<ChargeCode>> {
    let mut stmt = conn.prepare("SELECT * FROM charge_codes ORDER BY id")?;
    map_to_charge_codes(&mut stmt)
}

// Get active charge codes
pub fn get_active_charge_codes(conn: &Connection) -> Result<Vec<ChargeCode>> {
    let mut stmt = conn.prepare("SELECT * FROM charge_codes WHERE close IS NULL ORDER BY id")?;
    map_to_charge_codes(&mut stmt)
}

// Get active charge code ID
pub fn get_active_charge_code_id(conn: &Connection, code: &str) -> Result<i64> {
    conn.query_one(
        "SELECT id FROM charge_codes WHERE code = ? AND close IS NULL",
        params![code],
        |row| row.get("id"),
    )
}

// Add charge code
pub fn add_charge_code(conn: &Connection, code: &str, details: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO charge_codes (code, details, open) VALUES (?, ?, ?)",
        params![code, details, Utc::now()],
    )?;
    Ok(())
}

// Update charge code
pub fn update_charge_code(conn: &Connection, id: i64, details: &str) -> Result<usize> {
    conn.execute(
        "UPDATE charge_codes SET details = ? WHERE id = ?",
        params![details, id],
    )
}

// Delete charge code
pub fn delete_charge_code(conn: &Connection, id: i64) -> Result<usize> {
    conn.execute("DELETE FROM charge_codes WHERE id = ?", params![id])
}

// Close charge code
pub fn close_charge_code(conn: &Connection, code: &str) -> Result<()> {
    conn.execute(
        "UPDATE charge_codes SET close = ? WHERE code = ? AND close IS NULL",
        params![Utc::now(), code],
    )?;
    Ok(())
}
