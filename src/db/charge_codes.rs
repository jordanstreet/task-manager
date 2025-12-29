// Imports
use rusqlite::{Connection, Statement, params};
use chrono::{DateTime, Local};

// Charge code definition
#[derive(Debug, PartialEq)]
pub struct ChargeCode {
    pub id: i64,
    pub code: String,
    pub description: String,
    pub open: DateTime<Local>,
    pub close: Option<DateTime<Local>>,
}

// Execute an SQL query and return a vector of charge codes
fn map_to_charge_codes(stmt: &mut Statement) -> rusqlite::Result<Vec<ChargeCode>> {
    // Execute the query and map rows
    let rows = stmt.query_map([], |row| {
        Ok(ChargeCode {
            id: row.get("id")?,
            code: row.get("code")?,
            description: row.get("description")?,
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

// Add charge code
pub fn add_charge_code(conn: &Connection, code: &str, description: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO charge_codes (code, description, open) VALUES (?, ?, ?)",
        params![code, description, Local::now()],
    )?;
    Ok(())
}

// Close charge code
pub fn close_charge_code(conn: &Connection, code: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE charge_codes SET close = ? WHERE code = ? AND close IS NULL",
        params![Local::now(), code],
    )?;
    Ok(())
}

// Delete charge code
pub fn delete_charge_code(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM charge_codes WHERE id = ?", params![id])?;
    Ok(())
}

// Update charge code
pub fn update_charge_code(conn: &Connection, id: i64, description: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE charge_codes SET description = ? WHERE id = ?",
        params![description, id],
    )?;
    Ok(())
}
