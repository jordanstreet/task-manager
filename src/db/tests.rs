// Imports
use crate::db::test_utils::*;
use crate::db::{charge_codes::*, logs::*};
use rusqlite::{Error, ErrorCode};

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
    assert_eq!(
        add_charge_code(&conn, "CC1", "details 1")
            .unwrap_err()
            .sqlite_error()
            .unwrap()
            .code,
        ErrorCode::ConstraintViolation
    );

    // Close the existing CC1 and then add a newer version
    close_charge_code(&conn, "CC1").unwrap();
    add_charge_code(&conn, "CC1", "details 1").unwrap();

    // Update the CC2 details
    assert_eq!(
        update_charge_code(&conn, 2, "updated details 2").unwrap(),
        1
    );

    // There should be 3 total charge codes and 2 active
    let active_codes = get_active_charge_codes(&conn).unwrap();
    assert_eq!(get_all_charge_codes(&conn).unwrap().len(), 3);
    assert_eq!(active_codes.len(), 2);

    // Verify the charge code IDs
    assert_eq!(get_active_charge_code_id(&conn, "CC1").unwrap(), 3);
    assert_eq!(get_active_charge_code_id(&conn, "CC2").unwrap(), 2);

    // Retrieval of invalid charge code should error
    assert_eq!(
        get_active_charge_code_id(&conn, "sdf").unwrap_err(),
        Error::QueryReturnedNoRows
    );

    // Verify the two active charge codes
    assert_eq!(active_codes.len(), 2);
    assert_eq!(active_codes[0].id, 2);
    assert_eq!(active_codes[0].code, "CC2");
    assert_eq!(active_codes[0].details, "updated details 2");
    assert_eq!(active_codes[1].id, 3);
    assert_eq!(active_codes[1].code, "CC1");

    // Verify retrieval by ID
    assert_eq!(active_codes[0], get_charge_code(&conn, 2).unwrap());
    assert_eq!(active_codes[1], get_charge_code(&conn, 3).unwrap());

    // Retrieval by invalid ID should error
    assert_eq!(
        get_charge_code(&conn, 4).unwrap_err(),
        Error::QueryReturnedNoRows
    );
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
    assert_eq!(
        add_log_entry(
            &conn,
            "task 1",
            create_date_time("10:00"),
            Some(create_date_time("11:00")),
        )
        .unwrap(),
        1
    );
    link_charge_code(&conn, 1, 1).unwrap();
    link_charge_code(&conn, 1, 2).unwrap();
    assert_eq!(
        add_log_entry(&conn, "task 2", create_date_time("11:30"), None).unwrap(),
        2
    );
    link_charge_code(&conn, 2, 2).unwrap();

    // Linking a charge code twice should error
    assert_eq!(
        link_charge_code(&conn, 2, 2)
            .unwrap_err()
            .sqlite_error()
            .unwrap()
            .code,
        ErrorCode::ConstraintViolation
    );

    // Helper function to retrieve logs within a specified window
    let get_logs_between = |begin: &str, end: &str| {
        get_log_entries(&conn, create_date_time(begin), create_date_time(end)).unwrap()
    };

    // Empty range should return an empty vector without error
    assert_eq!(get_logs_between("01:00", "02:00").len(), 0);

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

    // Verify retrieval by ID
    assert_eq!(logs[0], get_log_entry(&conn, 1).unwrap());
    assert_eq!(logs[1], get_log_entry(&conn, 2).unwrap());

    // Retrieval by invalid ID should error
    assert_eq!(
        get_log_entry(&conn, 3).unwrap_err(),
        Error::QueryReturnedNoRows
    );

    // Retrieve a subset of logs
    assert_eq!(get_logs_between("09:00", "11:15").len(), 1);

    // Update the second log to start sooner and end
    assert_eq!(
        update_log_entry(&conn, 2, "task 2", create_date_time("11:05"), None).unwrap(),
        1
    );
    assert_eq!(stop_log(&conn).unwrap(), 1);

    // The same query should now return 2 logs, both with end times
    let logs2 = get_logs_between("09:00", "11:15");
    assert_eq!(logs2.len(), 2);
    assert!(logs2[1].stop.is_some());

    // Delete the second log entry and verify that there is now only 1
    assert_eq!(delete_log_entry(&conn, 2).unwrap(), 1);
    assert_eq!(get_logs_between("09:00", "11:15").len(), 1);

    // Relink to only CC2
    delete_log_entry_charge_codes(&conn, 1).unwrap();
    link_charge_code(&conn, 1, 2).unwrap();
    let logs3 = get_logs_between("09:00", "11:15");
    assert_eq!(logs3.len(), 1);
    assert_eq!(logs3[0].charge_codes, vec!["CC2"]);
}

// Charge code + log behavior
#[test]
fn charge_codes_logs() {
    // Setup
    let conn = setup();

    // Add a charge code that is replaced
    add_charge_code(&conn, "CC1", "details 1").unwrap();

    // Add a log that depends on CC1
    assert_eq!(
        add_log_entry(
            &conn,
            "task 1",
            create_date_time("09:00"),
            Some(create_date_time("10:00")),
        )
        .unwrap(),
        1
    );
    link_charge_code(&conn, 1, 1).unwrap();

    // Close the existing CC1 and then add a newer version
    close_charge_code(&conn, "CC1").unwrap();
    add_charge_code(&conn, "CC1", "details 1").unwrap();

    // Deleting the old CC1 should result in an error
    assert_eq!(
        delete_charge_code(&conn, 1)
            .unwrap_err()
            .sqlite_error()
            .unwrap()
            .code,
        ErrorCode::ConstraintViolation
    );

    // Remove the log that depends on the old CC1 and delete the old CC1
    assert_eq!(delete_log_entry(&conn, 1).unwrap(), 1);
    assert_eq!(delete_charge_code(&conn, 1).unwrap(), 1);
}
