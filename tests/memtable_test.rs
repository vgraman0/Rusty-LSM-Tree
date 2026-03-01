use rusty_lsm::memtable::{MemTable, SkipListMemTable};

#[test]
fn test_example() {
    let table = SkipListMemTable::new(4);

    assert!(table.is_empty());
    assert_eq!(table.len(), 0);
}

#[test]
fn put_and_get_single_key() {
    let mut table = SkipListMemTable::new(4);

    table.put(b"hello".to_vec(), b"world".to_vec());

    assert_eq!(table.len(), 1);
    assert_eq!(table.get(b"hello"), Some(b"world".as_slice()));
}

#[test]
fn put_overwrite_existing_key() {
    let mut table = SkipListMemTable::new(4);
    table.put(b"key".to_vec(), b"value1".to_vec());

    table.put(b"key".to_vec(), b"value2".to_vec());

    assert_eq!(table.get(b"key"), Some(b"value2".as_slice()));
    assert_eq!(table.len(), 1);
}

#[test]
fn get_nonexistent_key_returns_none() {
    let table = SkipListMemTable::new(4);

    assert_eq!(table.get(b"missing"), None);
}

#[test]
fn delete_existing_key() {
    let mut table = SkipListMemTable::new(4);
    table.put(b"key".to_vec(), b"value".to_vec());

    table.delete(b"key");

    assert_eq!(table.get(b"key"), None);
    assert_eq!(table.len(), 0);
}

#[test]
fn delete_nonexistent_key_does_nothing() {
    let mut table = SkipListMemTable::new(4);

    table.delete(b"missing");

    assert_eq!(table.len(), 0);
}

#[test]
fn delete_already_deleted_key() {
    let mut table = SkipListMemTable::new(4);
    table.put(b"key".to_vec(), b"value".to_vec());
    table.delete(b"key");

    table.delete(b"key");

    assert_eq!(table.len(), 0);
}

#[test]
fn put_after_delete_reinserts() {
    let mut table = SkipListMemTable::new(4);
    table.put(b"key".to_vec(), b"value1".to_vec());
    table.delete(b"key");

    table.put(b"key".to_vec(), b"value2".to_vec());

    assert_eq!(table.get(b"key"), Some(b"value2".as_slice()));
    assert_eq!(table.len(), 1);
}

#[test]
fn multiple_keys_all_retrievable() {
    let mut table = SkipListMemTable::new(4);

    table.put(b"cherry".to_vec(), b"3".to_vec());
    table.put(b"apple".to_vec(), b"1".to_vec());
    table.put(b"banana".to_vec(), b"2".to_vec());

    assert_eq!(table.get(b"apple"), Some(b"1".as_slice()));
    assert_eq!(table.get(b"banana"), Some(b"2".as_slice()));
    assert_eq!(table.get(b"cherry"), Some(b"3".as_slice()));
    assert_eq!(table.len(), 3);
}

#[test]
fn len_tracks_inserts_and_deletes() {
    let mut table = SkipListMemTable::new(4);
    assert_eq!(table.len(), 0);

    table.put(b"a".to_vec(), b"1".to_vec());
    assert_eq!(table.len(), 1);

    table.put(b"b".to_vec(), b"2".to_vec());
    assert_eq!(table.len(), 2);

    table.delete(b"a");
    assert_eq!(table.len(), 1);
}

#[test]
fn is_empty_reflects_state() {
    let mut table = SkipListMemTable::new(4);
    assert!(table.is_empty());

    table.put(b"a".to_vec(), b"1".to_vec());
    assert!(!table.is_empty());

    table.delete(b"a");
    assert!(table.is_empty());
}
