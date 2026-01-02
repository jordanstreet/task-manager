// Imports
use chrono::{DateTime, Utc};
use indexmap::{IndexMap, map::Entry};
use rusqlite::{Connection, Error, Result, params};

// Log definition
#[derive(Debug, PartialEq)]
pub struct LogEntry {
    pub id: i64,
    pub charge_codes: Vec<String>,
    pub description: String,
    pub start: DateTime<Utc>,
    pub stop: Option<DateTime<Utc>>,
}

// Get log entry by ID
pub fn get_log_entry(conn: &Connection, id: i64) -> Result<LogEntry> {
    // Create the SQL statement
    let mut stmt = conn.prepare(
        "
        SELECT
            l.*,
            cc.code as charge_code
        FROM logs l
        LEFT JOIN log_charge_codes lcc ON l.id = lcc.log_id
        LEFT JOIN charge_codes cc ON cc.id = lcc.charge_code_id
        WHERE l.id = ?
        ",
    )?;

    // Execute the query and create the log
    let mut rows = stmt.query(params![id])?;
    if let Some(row) = rows.next()? {
        // Create the log struct
        let mut log = LogEntry {
            id: row.get("id")?,
            charge_codes: vec![row.get("charge_code")?],
            description: row.get("description")?,
            start: row.get("start")?,
            stop: row.get("stop")?,
        };

        // Populate charge codes
        while let Some(row) = rows.next()? {
            log.charge_codes.push(row.get("charge_code")?);
        }
        Ok(log)
    } else {
        Err(Error::QueryReturnedNoRows)
    }
}

// Get log entries within time range
pub fn get_log_entries(
    conn: &Connection,
    begin: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<Vec<LogEntry>> {
    // Create the SQL statement
    let mut stmt = conn.prepare(
        "
        SELECT
            l.*,
            cc.code as charge_code
        FROM logs l
        LEFT JOIN log_charge_codes lcc ON l.id = lcc.log_id
        LEFT JOIN charge_codes cc ON cc.id = lcc.charge_code_id
        WHERE start < ?2 AND COALESCE(stop, ?2) > ?1
        ORDER BY id
        ",
    )?;

    // Execute the query and create the log vector
    let mut rows = stmt.query(params![begin, end])?;
    let mut results: IndexMap<i64, LogEntry> = IndexMap::new();
    while let Some(row) = rows.next()? {
        // Insert the log entry
        let entry = match results.entry(row.get("id")?) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(LogEntry {
                id: row.get("id")?,
                charge_codes: Vec::new(),
                description: row.get("description")?,
                start: row.get("start")?,
                stop: row.get("stop")?,
            }),
        };

        // Populate charge codes
        entry.charge_codes.push(row.get("charge_code")?);
    }

    // Return the results as a vector
    Ok(results.into_values().collect())
}

// Add log entry
pub fn add_log_entry(
    conn: &Connection,
    description: &str,
    start: DateTime<Utc>,
    stop: Option<DateTime<Utc>>,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO logs (description, start, stop) VALUES (?, ?, ?)",
        params![description, start, stop],
    )?;
    Ok(conn.last_insert_rowid())
}

// Add charge code link
pub fn link_charge_code(conn: &Connection, log_id: i64, charge_code_id: i64) -> Result<()> {
    conn.execute(
        "INSERT INTO log_charge_codes (log_id, charge_code_id) VALUES (?, ?)",
        params![log_id, charge_code_id],
    )?;
    Ok(())
}

// Update log entry
pub fn update_log_entry(
    conn: &Connection,
    id: i64,
    description: &str,
    start: DateTime<Utc>,
    stop: Option<DateTime<Utc>>,
) -> Result<usize> {
    conn.execute(
        "UPDATE logs SET description = ?, start = ?, stop = ? WHERE id = ?",
        params![description, start, stop, id],
    )
}

// Delete log entry
pub fn delete_log_entry(conn: &Connection, id: i64) -> Result<usize> {
    conn.execute("DELETE FROM logs WHERE id = ?", params![id])
}

// Delete log entry charge codes
pub fn delete_log_entry_charge_codes(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM log_charge_codes WHERE log_id = ?", params![id])?;
    Ok(())
}

// Stop log
pub fn stop_log(conn: &Connection) -> Result<usize> {
    conn.execute(
        "UPDATE logs SET stop = ? WHERE stop IS NULL",
        params![Utc::now()],
    )
}
