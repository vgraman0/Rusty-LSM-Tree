use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

use crate::error::{self, Result};

pub struct Wal {
    file: File,
}

impl Wal {
    pub fn create(path: &Path) -> Result<Self> {
        let file = File::create(path)?;
        Ok(Wal { file })
    }

    pub fn open(path: &Path) -> Result<Self> {
        let file = OpenOptions::new().append(true).open(path)?;
        Ok(Wal { file })
    }

    pub fn append(&mut self, key: &[u8], value: Option<&[u8]>) -> Result<()> {
        let tag = value.is_some() as u8;
        let key_len = key.len() as u32;

        self.file.write_all(&[tag])?;
        self.file.write_all(&key_len.to_le_bytes())?;
        self.file.write_all(key)?;
        if let Some(v) = value {
            let value_len = v.len() as u32;
            self.file.write_all(&value_len.to_le_bytes())?;
            self.file.write_all(v)?;
        }
        Ok(())
    }

    #[allow(clippy::type_complexity)]
    pub fn recover(path: &Path) -> Result<Vec<(Vec<u8>, Option<Vec<u8>>)>> {
        let mut entries = Vec::new();
        let mut file = File::open(path)?;

        loop {
            match Self::read_entry(&mut file) {
                Ok(entry) => entries.push(entry),
                Err(error::Error::Io(ref e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    break
                }
                Err(e) => return Err(e),
            }
        }

        Ok(entries)
    }

    fn read_entry(file: &mut File) -> Result<(Vec<u8>, Option<Vec<u8>>)> {
        let mut tag_buf = [0u8; 1];
        file.read_exact(&mut tag_buf)?;
        let tag = tag_buf[0] == 1;

        let mut key_len_buf = [0u8; 4];
        file.read_exact(&mut key_len_buf)?;
        let key_len = u32::from_le_bytes(key_len_buf);

        let mut key = vec![0u8; key_len as usize];
        file.read_exact(&mut key)?;

        let value = if tag {
            let mut val_len_buf = [0u8; 4];
            file.read_exact(&mut val_len_buf)?;
            let val_len = u32::from_le_bytes(val_len_buf);

            let mut value = vec![0u8; val_len as usize];
            file.read_exact(&mut value)?;
            Some(value)
        } else {
            None
        };

        Ok((key, value))
    }

    pub fn sync(&mut self) -> Result<()> {
        Ok(self.file.sync_all()?)
    }
}
