// Imports
use crate::db::{charge_codes::*, logs::*};
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use thiserror::Error;

// Errors
#[derive(Debug, Error, PartialEq)]
pub enum DBError {
    #[error("SQLite error: {0}")]
    SQLite(#[from] rusqlite::Error),
    #[error("unknown active charge code '{0}'")]
    UnknownActiveChargeCode(String),
    #[error("unknown charge code id {0}")]
    UnknownChargeCodeId(i64),
    #[error("charge code id {0} is in use")]
    ChargeCodeInUse(i64),
    #[error("unknown log id {0}")]
    UnknownLogId(i64),
    #[error("no running log")]
    NoRunningLog(),
    #[error("invalid start time {0}")]
    InvalidStartTime(DateTime<Utc>),
    #[error("invalid end time {0}")]
    InvalidEndTime(DateTime<Utc>),
    #[error("invalid time range {0} to {1}")]
    InvalidTimeRange(DateTime<Utc>, DateTime<Utc>),
    #[error("invalid arguments: {0}")]
    InvalidArguments(String),
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
        Ok(Database { conn })
    }

    // Get charge code by ID
    pub fn get_charge_code(&self, id: i64) -> Result<ChargeCode> {
        let charge_code = get_charge_code(&self.conn, id).map_err(|err| match err {
            rusqlite::Error::QueryReturnedNoRows => DBError::UnknownChargeCodeId(id),
            _ => DBError::SQLite(err),
        })?;
        Ok(charge_code)
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
    pub fn get_active_charge_code_id(&self, code: &str) -> Result<i64> {
        let id = get_active_charge_code_id(&self.conn, code).map_err(|err| match err {
            rusqlite::Error::QueryReturnedNoRows => {
                DBError::UnknownActiveChargeCode(String::from(code))
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

    // Update charge code
    pub fn update_charge_code(&self, id: i64, details: &str) -> Result<()> {
        if update_charge_code(&self.conn, id, details)? == 1 {
            Ok(())
        } else {
            Err(DBError::UnknownChargeCodeId(id))
        }
    }

    // Delete charge code
    pub fn delete_charge_code(&self, id: i64) -> Result<()> {
        // Delete the charge code and map constraint violation errors to charge code in use
        let n_deleted = delete_charge_code(&self.conn, id).map_err(|err| match err {
            rusqlite::Error::SqliteFailure(sql_err, _) => match sql_err.code {
                rusqlite::ErrorCode::ConstraintViolation => DBError::ChargeCodeInUse(id),
                _ => DBError::SQLite(err),
            },
            _ => DBError::SQLite(err),
        })?;

        // Return an error if a charge code wasn't deleted
        if n_deleted != 1 {
            Err(DBError::UnknownChargeCodeId(id))
        } else {
            Ok(())
        }
    }

    // Get log entry by ID
    pub fn get_log_entry(&self, id: i64) -> Result<LogEntry> {
        let entry = get_log_entry(&self.conn, id).map_err(|err| match err {
            rusqlite::Error::QueryReturnedNoRows => DBError::UnknownLogId(id),
            _ => DBError::SQLite(err),
        })?;
        Ok(entry)
    }

    // Get log entries within a time window
    pub fn get_log_entries(
        &self,
        begin: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<LogEntry>> {
        // Ensure time range is valid
        if end <= begin {
            return Err(DBError::InvalidTimeRange(begin, end));
        }

        // Return the log entries
        Ok(get_log_entries(&self.conn, begin, end)?)
    }

    // Check time range
    fn check_time_range(
        start: Option<DateTime<Utc>>,
        stop: Option<DateTime<Utc>>,
    ) -> Result<DateTime<Utc>> {
        // Get the current time
        let now = Utc::now();

        // Start and stop cannot be in the future
        if let Some(start) = start
            && start > now
        {
            return Err(DBError::InvalidStartTime(start));
        }
        if let Some(stop) = stop
            && stop > now
        {
            return Err(DBError::InvalidEndTime(stop));
        }

        // Stop must be after start
        if let (Some(start), Some(stop)) = (start, stop)
            && stop <= start
        {
            return Err(DBError::InvalidTimeRange(start, stop));
        };

        // Stop cannot be specified without start
        if start.is_none() && stop.is_some() {
            return Err(DBError::InvalidArguments(String::from(
                "'stop' cannot be specified without 'start'",
            )));
        }
        Ok(now)
    }

    // Adjust surrounding logs
    fn adjust_logs(
        &self,
        id: i64,
        start: DateTime<Utc>,
        stop: Option<DateTime<Utc>>,
        now: DateTime<Utc>,
    ) -> Result<()> {
        // Retrieve logs within the time window
        let stop = stop.unwrap_or(now);
        let logs = get_log_entries(&self.conn, start, stop)?;

        // Consider each log
        for log in logs {
            // Skip the working log
            if log.id == id {
                continue;
            }

            // Extract the log start and stop times
            let log_start = log.start;
            let log_stop = log.stop.unwrap_or(now);

            // Check time windows
            if log_start >= start && log_stop <= stop {
                // Delete logs that are completely consumed
                delete_log_entry(&self.conn, log.id)?;
            } else if log_start < start && log_stop > stop {
                // Split logs that started before and ended after
                update_log_entry(&self.conn, log.id, &log.description, log_start, Some(start))?;
                let new_id = add_log_entry(&self.conn, &log.description, stop, log.stop)?;
                for code in log.charge_codes {
                    link_charge_code(&self.conn, new_id, get_active_charge_code_id(&self.conn, &code)?)?;
                }
            } else if log_start < start && log_stop > start {
                // Trim logs that started before and ended within
                update_log_entry(&self.conn, log.id, &log.description, log_start, Some(start))?;
            } else if log_start < stop && log_stop > stop {
                // Trim logs that started within and ended after
                update_log_entry(&self.conn, log.id, &log.description, stop, log.stop)?;
            }
        }
        Ok(())
    }

    // Add a log entry
    pub fn add_log_entry(
        &self,
        description: &str,
        charge_codes: Vec<String>,
        start: Option<DateTime<Utc>>,
        stop: Option<DateTime<Utc>>,
    ) -> Result<()> {
        // Ensure time range is valid
        let now = Self::check_time_range(start, stop)?;

        // Verify each charge code
        for code in charge_codes.clone() {
            self.get_active_charge_code_id(&code)?;
        }

        // Stop any existing logs
        if start.is_none() {
            stop_log(&self.conn)?;
        }

        // Add the log entry
        let id = add_log_entry(&self.conn, description, start.unwrap_or(now), stop)?;

        // Link each charge code
        for code in charge_codes {
            link_charge_code(&self.conn, id, self.get_active_charge_code_id(&code)?)?
        }

        // Adjust affected logs
        if let Some(start) = start {
            self.adjust_logs(id, start, stop, now)?;
        }
        Ok(())
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
        let now = Self::check_time_range(Some(start), stop)?;

        // Verify each charge code
        for code in charge_codes.clone() {
            self.get_active_charge_code_id(&code)?;
        }

        // Update the log
        if update_log_entry(&self.conn, id, description, start, stop)? != 1 {
            return Err(DBError::UnknownLogId(id));
        }

        // Delete existing linked charge codes
        delete_log_entry_charge_codes(&self.conn, id)?;

        // Link each charge code
        for code in charge_codes {
            link_charge_code(&self.conn, id, self.get_active_charge_code_id(&code)?)?
        }

        // Adjust affected logs
        self.adjust_logs(id, start, stop, now)?;
        Ok(())
    }

    // Delete log entry
    pub fn delete_log_entry(&self, id: i64) -> Result<()> {
        if delete_log_entry(&self.conn, id)? == 1 {
            Ok(())
        } else {
            Err(DBError::UnknownLogId(id))
        }
    }

    // Stop log
    pub fn stop_log(&self) -> Result<()> {
        if stop_log(&self.conn)? < 1 {
            Err(DBError::NoRunningLog())
        } else {
            Ok(())
        }
    }
}
