// Imports
use crate::db::{charge_codes::*, logs::*};
use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc};
use rusqlite::Connection;

// Create an in-memory test DB
fn setup() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute("PRAGMA foreign_keys = ON;", []).unwrap();
    conn.execute_batch(include_str!("create_tables.sql"))
        .unwrap();
    conn
}

// Charge code behavior
#[test]
fn charge_codes() {
    // Setup
    let conn = setup();

    // Empty queries should return empty vectors without error
    assert_eq!(get_all_charge_codes(&conn).unwrap().len(), 0);
    assert_eq!(get_active_charge_codes(&conn).unwrap().len(), 0);

    // Add a couple of charge codes
    add_charge_code(&conn, "CC1", "details 1").unwrap();
    add_charge_code(&conn, "CC2", "details 2").unwrap();

    // Adding a duplicate charge code should error
    assert!(
        add_charge_code(&conn, "CC1", "details 1")
            .unwrap_err()
            .sqlite_error()
            .unwrap()
            .code
            == rusqlite::ErrorCode::ConstraintViolation
    );

    // Close the existing CC1 and then add a newer version
    close_charge_code(&conn, "CC1").unwrap();
    add_charge_code(&conn, "CC1", "details 1").unwrap();

    // Update the CC2 details
    assert_eq!(update_charge_code_details(&conn, 2, "updated details 2").unwrap(), 1);

    // There should be 3 total charge codes and 2 active
    let active_codes = get_active_charge_codes(&conn).unwrap();
    assert_eq!(get_all_charge_codes(&conn).unwrap().len(), 3);
    assert_eq!(active_codes.len(), 2);

    // Verify the charge code IDs
    assert_eq!(get_charge_code_id(&conn, "CC1").unwrap(), 3);
    assert_eq!(get_charge_code_id(&conn, "CC2").unwrap(), 2);

    // Verify the two active charge codes
    assert_eq!(active_codes.len(), 2);
    assert_eq!(active_codes[0].id, 2);
    assert_eq!(active_codes[0].code, "CC2");
    assert_eq!(active_codes[0].details, "updated details 2");
    assert_eq!(active_codes[1].id, 3);
    assert_eq!(active_codes[1].code, "CC1");
}

// Helper function to create a DateTime
fn create_date_time(str: &str) -> DateTime<Utc> {
    let time = NaiveTime::parse_from_str(str, "%H:%M").unwrap();
    let date_time = NaiveDate::from_ymd_opt(2025, 01, 01)
        .unwrap()
        .and_time(time);
    Utc.from_utc_datetime(&date_time)
}

// Log behavior
#[test]
fn logs() {
    // Setup
    let conn = setup();

    // Add a couple of charge codes
    add_charge_code(&conn, "CC1", "details 1").unwrap();
    add_charge_code(&conn, "CC2", "details 2").unwrap();

    // Add a couple logs with charge codes
    assert_eq!(add_log_entry(
        &conn,
        "task 1",
        create_date_time("10:00"),
        Some(create_date_time("11:00")),
    )
    .unwrap(), 1);
    link_charge_code(&conn, 1, 1).unwrap();
    link_charge_code(&conn, 1, 2).unwrap();
    assert_eq!(add_log_entry(&conn, "task 2", create_date_time("11:30"), None).unwrap(), 2);
    link_charge_code(&conn, 2, 2).unwrap();

    // Helper function to retrieve logs within a specified window
    let get_logs_between = |begin: &str, end: &str| {
        get_log_entries(&conn, create_date_time(begin), create_date_time(end)).unwrap()
    };

    // Retrieve logs
    let logs = get_logs_between("10:30", "12:00");

    // Verify logs
    assert_eq!(logs.len(), 2);
    assert_eq!(logs[0].id, 1);
    assert_eq!(logs[0].description, "task 1");
    assert_eq!(logs[0].charge_codes, vec!["CC1", "CC2"]);
    assert_eq!(logs[1].id, 2);
    assert_eq!(logs[1].description, "task 2");
    assert_eq!(logs[1].charge_codes, vec!["CC2"]);

    // Retrieve a subset of logs
    assert_eq!(get_logs_between("09:00", "11:15").len(), 1);

    // Update the second log to start sooner and end
    assert_eq!(update_log_entry(&conn, 2, "task 2", create_date_time("11:05"), None).unwrap(), 1);
    assert_eq!(stop_log(&conn).unwrap(), 1);

    // The same query should now return 2 logs, both with end times
    let logs2 = get_logs_between("09:00", "11:15");
    assert_eq!(logs2.len(), 2);
    assert!(logs2[1].stop.is_some());

    // Delete the first log entry and verify that there is now only 1
    delete_log_entry(&conn, 1).unwrap();
    assert_eq!(get_logs_between("09:00", "11:15").len(), 1);
}

// Charge code + log behavior
#[test]
fn charge_codes_logs() {
    // Setup
    let conn = setup();

    // Add a charge code that is replaced
    add_charge_code(&conn, "CC1", "details 1").unwrap();

    // Add a log that depends on CC1
    assert_eq!(add_log_entry(
        &conn,
        "task 1",
        create_date_time("09:00"),
        Some(create_date_time("10:00")),
    )
    .unwrap(), 1);
    link_charge_code(&conn, 1, 1).unwrap();

    // Close the existing CC1 and then add a newer version
    close_charge_code(&conn, "CC1").unwrap();
    add_charge_code(&conn, "CC1", "details 1").unwrap();

    // Deleting the old CC1 should result in an error
    assert!(
        delete_charge_code(&conn, 1)
            .unwrap_err()
            .sqlite_error()
            .unwrap()
            .code
            == rusqlite::ErrorCode::ConstraintViolation
    );

    // Remove the log that depends on the old CC1 and delete the old CC1
    delete_log_entry(&conn, 1).unwrap();
    assert_eq!(delete_charge_code(&conn, 1).unwrap(), 1);
}
