use rusty_lsm::error;
use rusty_lsm::wal::Wal;

#[test]
fn recovered_entry_matches_wal() -> error::Result<()> {
    let test_dir = tempfile::tempdir()?;
    let wal_path = test_dir.path().join("test.wal");

    let mut wal = Wal::create(&wal_path)?;
    wal.append(b"key", Some(b"val"))?;

    let entries = Wal::recover(&wal_path)?;
    assert_eq!(entries.len(), 1);
    let (key, value) = &entries[0];
    assert_eq!(key, &b"key");
    assert_eq!(value, &Some(b"val".to_vec()));

    Ok(())
}

#[test]
fn open_appends_to_existing_wal() -> error::Result<()> {
    let test_dir = tempfile::tempdir()?;
    let wal_path = test_dir.path().join("test.wal");

    {
        let mut wal = Wal::create(&wal_path)?;
        wal.append(b"key", Some(b"val"))?;
    }

    {
        let mut wal = Wal::open(&wal_path)?;
        wal.append(b"key2", Some(b"val2"))?;
    }

    let entries = Wal::recover(&wal_path)?;
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0], (b"key".to_vec(), Some(b"val".to_vec())));
    assert_eq!(entries[1], (b"key2".to_vec(), Some(b"val2".to_vec())));

    Ok(())
}
