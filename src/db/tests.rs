// Imports
use crate::db::charge_codes::*;
use chrono::Local;
use rusqlite::{Connection, params};

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
pub fn charge_codes() {
    // Setup
    let conn = setup();

    // Empty queries should return empty vectors without error
    assert_eq!(get_all_charge_codes(&conn).unwrap().len(), 0);
    assert_eq!(get_active_charge_codes(&conn).unwrap().len(), 0);

    // Add a couple charge codes
    add_charge_code(&conn, "CC1", "description 1").unwrap();
    add_charge_code(&conn, "CC2", "description 2").unwrap();

    // Adding a duplicate charge code should error
    assert!(
        add_charge_code(&conn, "CC1", "description 1")
            .unwrap_err()
            .sqlite_error()
            .unwrap()
            .extended_code
            == rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE
    );

    // Close the existing CC1 and then add a newer version
    close_charge_code(&conn, "CC1").unwrap();
    add_charge_code(&conn, "CC1", "description 1").unwrap();

    // Update the CC2 description
    update_charge_code(&conn, 2, "updated description 2").unwrap();

    // There should be 3 total charge codes and 2 active
    let active_codes = get_active_charge_codes(&conn).unwrap();
    assert_eq!(get_all_charge_codes(&conn).unwrap().len(), 3);
    assert_eq!(active_codes.len(), 2);

    // Verify the two active charge codes
    assert_eq!(active_codes[0].id, 2);
    assert_eq!(active_codes[0].code, "CC2");
    assert_eq!(active_codes[0].description, "updated description 2");
    assert_eq!(active_codes[1].id, 3);
    assert_eq!(active_codes[1].code, "CC1");

    // Add a project that depends on the old CC1
    conn.execute(
        "INSERT INTO projects (title, description, default_charge_code, open) VALUES (?, ?, ?, ?)",
        params!["Project", "Example project", 1, Local::now()],
    )
    .unwrap();

    // Deleting the old CC1 should result in an error
    assert!(
        delete_charge_code(&conn, 1)
            .unwrap_err()
            .sqlite_error()
            .unwrap()
            .extended_code
            == rusqlite::ffi::SQLITE_CONSTRAINT_FOREIGNKEY
    );

    // Remove the project that depends on the old CC1 and delete the old CC1
    conn.execute("DELETE FROM projects WHERE id = ?", params![1]).unwrap();
    delete_charge_code(&conn, 1).unwrap();

    // Now there should only be 2 charge codes
    assert_eq!(get_all_charge_codes(&conn).unwrap().len(), 2);
}
