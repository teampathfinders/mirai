use crate::database::Database;

#[test]
fn database_open() {
    let _ = Database::new("test/db").unwrap();
}
