// Imports
use crate::service::db::*;

#[test]
fn test() {
    // Create an in-memory database
    let db = Database::new(None).unwrap();

    let _ = db.delete_charge_code(1).map_err(|err| println!("{}", err));
}
