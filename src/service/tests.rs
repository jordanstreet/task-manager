use chrono::{Duration, Utc};

// Imports
use crate::db::test_utils::*;
use crate::service::db::*;

// Charge code behavior
#[test]
fn charge_codes() {
    // Create a test database
    let db = Database::new(None).unwrap();

    // Add a couple charge codes
    db.add_charge_code("CC1", "details 1").unwrap();
    db.add_charge_code("CC2", "details 2").unwrap();

    // Get the ID of CC1
    let id = db.get_active_charge_code_id("CC1").unwrap();
    assert_eq!(id, 1);

    // Retrieving the ID of an unknown charge code should error
    assert_eq!(
        db.get_active_charge_code_id("UNKNOWN").unwrap_err(),
        DBError::UnknownActiveChargeCode(String::from("UNKNOWN"))
    );

    // Add a new CC1
    db.add_charge_code("CC1", "updated details 1").unwrap();

    // Check the new id
    assert_eq!(db.get_active_charge_code_id("CC1").unwrap(), 3);

    // At this point there are 3 charge codes (2 active)
    let codes = db.get_charge_codes(false).unwrap();
    assert_eq!(codes.len(), 3);
    assert_eq!(db.get_charge_codes(true).unwrap().len(), 2);

    // Verify retrieval by ID
    assert_eq!(db.get_charge_code(1).unwrap(), codes[0]);
    assert_eq!(db.get_charge_code(2).unwrap(), codes[1]);
    assert_eq!(db.get_charge_code(3).unwrap(), codes[2]);

    // Retrieval by an unknown ID should error
    assert_eq!(
        db.get_charge_code(4).unwrap_err(),
        DBError::UnknownChargeCodeId(4)
    );

    // Update a charge code
    db.update_charge_code(2, "updated details 2").unwrap();
    assert_eq!(db.get_charge_code(2).unwrap().details, "updated details 2");

    // Update of an unknown charge code ID should error
    assert_eq!(
        db.update_charge_code(4, "").unwrap_err(),
        DBError::UnknownChargeCodeId(4)
    );

    // Delete a charge code
    db.delete_charge_code(1).unwrap();
    assert_eq!(
        db.get_charge_code(1).unwrap_err(),
        DBError::UnknownChargeCodeId(1)
    );

    // Deleting an unknown charge code ID should error
    assert_eq!(
        db.delete_charge_code(4).unwrap_err(),
        DBError::UnknownChargeCodeId(4)
    );

    // Deleting a linked charge code should error
    db.add_log_entry("", vec![String::from("CC1")], None, None)
        .unwrap();
    assert_eq!(
        db.delete_charge_code(3).unwrap_err(),
        DBError::ChargeCodeInUse(3)
    );
}

// Log behavior
#[test]
fn logs() {
    // Create a test database
    let db = Database::new(None).unwrap();

    // Add a couple charge codes
    db.add_charge_code("CC", "details").unwrap();
    db.add_charge_code("CC2", "details 2").unwrap();

    // Add a couple log entries
    db.add_log_entry(
        "task 1",
        vec![String::from("CC")],
        Some(create_date_time("09:00")),
        Some(create_date_time("10:00")),
    )
    .unwrap();
    db.add_log_entry(
        "task 2",
        vec![String::from("CC")],
        Some(create_date_time("10:00")),
        Some(create_date_time("11:00")),
    )
    .unwrap();

    // Retrieve logs
    let get_logs = || {
        db.get_log_entries(create_date_time("01:00"), create_date_time("12:00"))
            .unwrap()
    };
    let mut logs = get_logs();
    assert_eq!(logs.len(), 2);

    // An invalid time window should error
    let bad_start = create_date_time("09:00");
    let bad_end = create_date_time("08:00");
    assert_eq!(
        db.get_log_entries(bad_start, bad_end).unwrap_err(),
        DBError::InvalidTimeRange(bad_start, bad_end)
    );

    // Verify retrieval by ID
    assert_eq!(db.get_log_entry(1).unwrap(), logs[0]);
    assert_eq!(db.get_log_entry(2).unwrap(), logs[1]);

    // Retrieval by an unknown ID should error
    assert_eq!(db.get_log_entry(3).unwrap_err(), DBError::UnknownLogId(3));

    // Attempting to stop while no logs are running should error
    assert_eq!(db.stop_log().unwrap_err(), DBError::NoRunningLog());

    // Start a new log without an end
    db.add_log_entry(
        "task 3",
        vec![String::from("CC")],
        Some(create_date_time("11:00")),
        None,
    )
    .unwrap();
    assert!(db.get_log_entry(3).unwrap().stop.is_none());

    // Stop the running log
    db.stop_log().unwrap();
    assert!(db.get_log_entry(3).unwrap().stop.is_some());

    // Delete the last log
    db.delete_log_entry(3).unwrap();
    assert_eq!(db.get_log_entry(3).unwrap_err(), DBError::UnknownLogId(3));

    // Attempting to delete an unknown log should error
    assert_eq!(
        db.delete_log_entry(3).unwrap_err(),
        DBError::UnknownLogId(3)
    );

    // Adding a log with an invalid start time should error
    let invalid_start = Utc::now() + Duration::minutes(10);
    assert_eq!(
        db.add_log_entry(
            "task 4",
            vec![String::from("CC")],
            Some(invalid_start),
            None
        )
        .unwrap_err(),
        DBError::InvalidStartTime(invalid_start)
    );

    // Adding a log with an invalid end time should error
    let valid_start = Utc::now() - Duration::minutes(10);
    let invalid_end = Utc::now() + Duration::minutes(10);
    assert_eq!(
        db.add_log_entry(
            "task 4",
            vec![String::from("CC")],
            Some(valid_start),
            Some(invalid_end)
        )
        .unwrap_err(),
        DBError::InvalidEndTime(invalid_end)
    );

    // Adding a log with start after stop should error
    assert_eq!(
        db.add_log_entry(
            "task 4",
            vec![String::from("CC")],
            Some(bad_start),
            Some(bad_end)
        )
        .unwrap_err(),
        DBError::InvalidTimeRange(bad_start, bad_end)
    );

    // Specifying stop without a start should error
    assert_eq!(
        db.add_log_entry(
            "task 4",
            vec![String::from("CC")],
            None,
            Some(create_date_time("09:00"))
        )
        .unwrap_err(),
        DBError::InvalidArguments(String::from("'stop' cannot be specified without 'start'"))
    );

    // Invalid charge codes should error
    assert_eq!(
        db.add_log_entry("task 4", vec![String::from("BAD")], None, None)
            .unwrap_err(),
        DBError::UnknownActiveChargeCode(String::from("BAD"))
    );

    // Update an existing log
    assert_eq!(logs[1].charge_codes, vec![String::from("CC")]);
    db.update_log_entry(
        2,
        "task 2 - updated",
        vec![String::from("CC2")],
        logs[1].start,
        logs[1].stop,
    )
    .unwrap();
    logs = get_logs();
    assert_eq!(logs[1].charge_codes, vec![String::from("CC2")]);
    assert_eq!(logs[1].description, "task 2 - updated");

    // Logs that are completely consumed should be deleted
    db.add_log_entry(
        "task 4",
        vec![String::from("CC")],
        Some(create_date_time("08:30")),
        Some(create_date_time("11:30")),
    )
    .unwrap();
    logs = get_logs();
    assert_eq!(logs.len(), 1);

    // Inserting within a log should split it
    db.add_log_entry(
        "task 5",
        vec![String::from("CC2")],
        Some(create_date_time("09:00")),
        Some(create_date_time("11:00")),
    )
    .unwrap();
    logs = get_logs();
    assert_eq!(logs.len(), 3);
    assert_eq!(logs[0].description, "task 4");
    assert_eq!(logs[0].charge_codes, vec![String::from("CC")]);
    assert_eq!(logs[0].start, create_date_time("08:30"));
    assert_eq!(logs[0].stop.unwrap(), create_date_time("09:00"));
    assert_eq!(logs[1].description, "task 5");
    assert_eq!(logs[1].charge_codes, vec![String::from("CC2")]);
    assert_eq!(logs[1].start, create_date_time("09:00"));
    assert_eq!(logs[1].stop.unwrap(), create_date_time("11:00"));
    assert_eq!(logs[2].description, "task 4");
    assert_eq!(logs[2].charge_codes, vec![String::from("CC")]);
    assert_eq!(logs[2].start, create_date_time("11:00"));
    assert_eq!(logs[2].stop.unwrap(), create_date_time("11:30"));

    // Adding a log before that ended within should trim the existing log start time
    db.add_log_entry(
        "task 6",
        vec![String::from("CC")],
        Some(create_date_time("08:00")),
        Some(create_date_time("08:45")),
    )
    .unwrap();
    logs = get_logs();
    assert_eq!(logs.len(), 4);
    assert_eq!(logs[0].description, "task 6");
    assert_eq!(logs[0].start, create_date_time("08:00"));
    assert_eq!(logs[0].stop.unwrap(), create_date_time("08:45"));
    assert_eq!(logs[1].description, "task 4");
    assert_eq!(logs[1].start, create_date_time("08:45"));
    assert_eq!(logs[1].stop.unwrap(), create_date_time("09:00"));

    // Adding a log after that started within should trim the existing log stop time
    db.add_log_entry(
        "task 7",
        vec![String::from("CC")],
        Some(create_date_time("11:15")),
        Some(create_date_time("11:45")),
    )
    .unwrap();
    logs = get_logs();
    assert_eq!(logs.len(), 5);
    assert_eq!(logs[3].description, "task 4");
    assert_eq!(logs[3].start, create_date_time("11:00"));
    assert_eq!(logs[3].stop.unwrap(), create_date_time("11:15"));
    assert_eq!(logs[4].description, "task 7");
    assert_eq!(logs[4].start, create_date_time("11:15"));
    assert_eq!(logs[4].stop.unwrap(), create_date_time("11:45"));
}
