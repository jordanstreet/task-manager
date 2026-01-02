// Imports
use crate::db::{charge_codes::*, logs::*};
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use thiserror::Error;

// Errors
#[derive(Debug, Error)]
pub enum DBError {
    #[error("SQLite error: {0}")]
    SQLite(#[from] rusqlite::Error),
    #[error("Unknown active charge code '{0}'")]
    UnknownActiveChargeCode(String),
    #[error("Unknown charge code id {0}")]
    UnknownChargeCodeId(i64),
    #[error("No running log")]
    NoRunningLog(),
    #[error("Invalid time range {0} to {1}")]
    InvalidTimeRange(DateTime<Utc>, DateTime<Utc>),
    #[error("Unknown log id {0}")]
    UnknownLogId(i64),
}

// Custom result type
pub type Result<T> = std::result::Result<T, DBError>;

// Database struct
pub struct Database {
    conn: rusqlite::Connection,
}

// Database methods
impl Database {
    // Open database and enable foreign keys
    pub fn new(db_path: Option<&str>) -> Result<Database> {
        // Establish a connection
        let conn = match db_path {
            Some(path) => Connection::open(path),
            None => Connection::open_in_memory(),
        }?;

        // Enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON;", [])?;

        // Setup tables
        conn.execute_batch(include_str!("../db/create_tables.sql"))?;

        // Return the database struct
        Ok(Database { conn: conn })
    }

    // Get charge codes
    pub fn get_charge_codes(&self, active_only: bool) -> Result<Vec<ChargeCode>> {
        Ok(if active_only {
            get_active_charge_codes(&self.conn)?
        } else {
            get_all_charge_codes(&self.conn)?
        })
    }

    // Get active charge code ID
    pub fn get_charge_code_id(&self, code: &str) -> Result<i64> {
        let id = get_charge_code_id(&self.conn, code).map_err(|err| match err {
            rusqlite::Error::QueryReturnedNoRows => {
                DBError::UnknownActiveChargeCode(code.to_string())
            }
            _ => DBError::SQLite(err),
        })?;
        Ok(id)
    }

    // Add charge code
    pub fn add_charge_code(&self, code: &str, details: &str) -> Result<()> {
        // Close out any existing duplicates
        close_charge_code(&self.conn, code)?;

        // Add the charge code
        add_charge_code(&self.conn, code, details)?;
        Ok(())
    }

    // Delete charge code
    pub fn delete_charge_code(&self, id: i64) -> Result<()> {
        if delete_charge_code(&self.conn, id)? == 1 {
            Ok(())
        } else {
            Err(DBError::UnknownChargeCodeId(id))
        }
    }

    // Update charge code details
    pub fn update_charge_code_details(&self, id: i64, details: &str) -> Result<()> {
        if update_charge_code_details(&self.conn, id, details)? == 1 {
            Ok(())
        } else {
            Err(DBError::UnknownChargeCodeId(id))
        }
    }

    // Get log entries within a time window
    pub fn get_log_entries(
        &self,
        begin: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<LogEntry>> {
        Ok(get_log_entries(&self.conn, begin, end)?)
    }

    // Add a log entry
    pub fn add_log_entry(
        &self,
        description: &str,
        charge_codes: Vec<String>,
        start: Option<DateTime<Utc>>,
        stop: Option<DateTime<Utc>>,
    ) -> Result<()> {
        // Verify each charge code
        for code in charge_codes.clone() {
            self.get_charge_code_id(&code)?;
        }

        // Stop any existing logs
        stop_log(&self.conn)?;

        // Add the log entry
        let log_id = add_log_entry(&self.conn, description, start.unwrap_or(Utc::now()), stop)?;

        // Link each charge code
        for code in charge_codes {
            link_charge_code(&self.conn, log_id, self.get_charge_code_id(&code)?)?
        }
        Ok(())
    }

    // Stop log
    pub fn stop_log(&self) -> Result<()> {
        if stop_log(&self.conn)? != 1 {
            Err(DBError::NoRunningLog())
        } else {
            Ok(())
        }
    }

    // Update log entry
    pub fn update_log_entry(
        &self,
        id: i64,
        description: &str,
        charge_codes: Vec<String>,
        start: DateTime<Utc>,
        stop: Option<DateTime<Utc>>,
    ) -> Result<()> {
        // Ensure time range is valid
        if let Some(stop) = stop {
            if stop <= start {
                return Err(DBError::InvalidTimeRange(start, stop));
            }
        };

        // Verify each charge code
        for code in charge_codes {
            self.get_charge_code_id(&code)?;
        }

        // Update the log
        if update_log_entry(&self.conn, id, description, start, stop)? != 1 {
            Err(DBError::UnknownLogId(id))
        } else {
            Ok(())
        }
    }
}
