use rs_sqlite::{Row, Table, get_row_slice, handle_input};
use std::fs;

// Helper to guarantee an isolated, clean database environment per test execution
fn setup_clean_db(db_name: &str) -> Table {
    let _ = fs::remove_file(db_name);
    Table::db_open(db_name).unwrap()
}

#[test]
fn test_insert_and_retrieve_row() {
    let db_path = "test_insert.db";
    let mut table = setup_clean_db(db_path);

    let res = handle_input("insert 1 user1 user1@email.com", &mut table);
    assert_eq!(res, None);
    assert_eq!(table.num_rows, 1);

    let slice = get_row_slice(&mut table, 0);
    let row = Row::deserialize(slice);
    assert_eq!(row.id, 1);

    let _ = table.db_close();
    let _ = fs::remove_file(db_path);
}

#[test]
fn test_keeps_data_after_closing_connection() {
    let db_path = "test_persistence.db";
    let _ = fs::remove_file(db_path);

    {
        let mut table = Table::db_open(db_path).unwrap();
        let res = handle_input("insert 1 user1 person1@example.com", &mut table);
        assert_eq!(res, None);
        table.db_close().unwrap();
    }

    {
        let mut table = Table::db_open(db_path).unwrap();
        assert_eq!(table.num_rows, 1);

        let slice = get_row_slice(&mut table, 0);
        let row = Row::deserialize(slice);
        assert_eq!(row.id, 1);
        let _ = table.db_close();
    }

    let _ = fs::remove_file(db_path);
}

#[test]
fn test_table_full_error() {
    let db_path = "test_full.db";
    let mut table = setup_clean_db(db_path);

    for i in 1..=1400 {
        let input = format!("insert {} user{} user{}@email.com", i, i, i);
        assert_eq!(handle_input(&input, &mut table), None);
    }

    let overflow = "insert 1401 user1401 user1401@email.com";
    let res = handle_input(overflow, &mut table);
    assert_eq!(res, Some("Error: Table full.".to_string()));

    let _ = table.db_close();
    let _ = fs::remove_file(db_path);
}

#[test]
fn test_maximum_length_strings() {
    let db_path = "test_max_len.db";
    let mut table = setup_clean_db(db_path);

    let long_username = "a".repeat(32);
    let long_email = "a".repeat(255);
    let cmd = format!("insert 1 {} {}", long_username, long_email);

    assert_eq!(handle_input(&cmd, &mut table), None);

    let _ = table.db_close();
    let _ = fs::remove_file(db_path);
}

#[test]
fn test_strings_too_long() {
    let db_path = "test_too_long.db";
    let mut table = setup_clean_db(db_path);

    let long_username = "a".repeat(33);
    let long_email = "a".repeat(256);
    let cmd = format!("insert 1 {} {}", long_username, long_email);

    let res = handle_input(&cmd, &mut table);
    assert_eq!(res, Some("String is too long.".to_string()));

    let _ = table.db_close();
    let _ = fs::remove_file(db_path);
}

#[test]
fn test_negative_id_error() {
    let db_path = "test_neg_id.db";
    let mut table = setup_clean_db(db_path);
    let cmd = "insert -1 cstack foo@bar.com";

    let res = handle_input(cmd, &mut table);
    assert_eq!(res, Some("ID must be positive.".to_string()));

    let _ = table.db_close();
    let _ = fs::remove_file(db_path);
}
