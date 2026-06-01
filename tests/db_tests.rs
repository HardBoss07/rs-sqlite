use rs_sqlite::{Row, Table, get_row_slice, handle_input};

#[test]
fn test_insert_and_retrieve_row() {
    let mut table = Table::new();

    let res = handle_input("insert 1 user1 user1@email.com", &mut table);
    assert_eq!(res, None);
    assert_eq!(table.num_rows, 1);

    let slice = get_row_slice(&table, 0);
    let row = Row::deserialize(slice);
    assert_eq!(row.id, 1);
}

#[test]
fn test_table_full_error() {
    let mut table = Table::new();

    for i in 0..1400 {
        let input = format!("insert {} user{} user{}@email.com", i, i, i);
        assert_eq!(handle_input(&input, &mut table), None);
    }

    let overflow = "insert 1401 user1401 user1401@email.com";
    let res = handle_input(overflow, &mut table);
    assert_eq!(res, Some("Error: Table full.".to_string()));
}

#[test]
fn test_maximum_length_strings() {
    let mut table = Table::new();

    let long_username = "a".repeat(32);
    let long_email = "a".repeat(255);
    let cmd = format!("insert 1 {} {}", long_username, long_email);

    assert_eq!(handle_input(&cmd, &mut table), None);
}

#[test]
fn test_strings_too_long() {
    let mut table = Table::new();

    let long_username = "a".repeat(33);
    let long_email = "a".repeat(256);
    let cmd = format!("insert 1 {} {}", long_username, long_email);

    let res = handle_input(&cmd, &mut table);
    assert_eq!(res, Some("String is too long.".to_string()));
}

#[test]
fn test_negative_id_error() {
    let mut table = Table::new();
    let cmd = "insert -1 cstack foo@bar.com";

    let res = handle_input(cmd, &mut table);
    assert_eq!(res, Some("ID must be positive.".to_string()));
}
