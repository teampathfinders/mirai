use crate::database::Database;

#[test]
fn database_open() {
    let _ = Database::new("test/db").unwrap();
}

#[test]
fn database_load_key() {
    let _ = Database::new("test/db").unwrap();
}
