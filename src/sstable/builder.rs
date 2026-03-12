use std::io::Write;
use std::{fs::File, path::Path};

use super::{BLOCK_SIZE, SSTABLE_MAGIC};
use crate::bloom::BloomFilter;
use crate::error::Result;

pub struct SSTableBuilder {
    file: File,
    current_block: Vec<u8>,
    index_entries: Vec<(Vec<u8>, u64)>,
    bloom: BloomFilter,
    current_offset: u64,
    entry_count: usize,
    last_key: Vec<u8>,
}

impl SSTableBuilder {
    pub fn new(path: &Path, expected_entries: usize) -> Result<Self> {
        let file = File::create(path)?;
        let current_block = Vec::new();
        let index_entries = Vec::new();
        let bloom = BloomFilter::new(expected_entries, 0.01);
        let current_offset = 0;
        let entry_count = 0;
        let last_key = Vec::new();

        Ok(SSTableBuilder {
            file,
            current_block,
            index_entries,
            bloom,
            current_offset,
            entry_count,
            last_key,
        })
    }

    pub fn add(&mut self, key: &[u8], value: Option<&[u8]>) -> Result<()> {
        self.bloom.insert(key);

        let key_len = key.len() as u32;
        self.current_block.extend_from_slice(&key_len.to_le_bytes());
        self.current_block.extend_from_slice(key);
        self.last_key = key.to_vec();
        if let Some(v) = value {
            let value_len = v.len() as u32;
            self.current_block
                .extend_from_slice(&value_len.to_le_bytes());
            self.current_block.extend_from_slice(v);
        } else {
            let value_len = u32::MAX;
            self.current_block
                .extend_from_slice(&value_len.to_le_bytes());
        }
        self.entry_count += 1;

        if self.current_block.len() >= BLOCK_SIZE {
            let block_start = self.current_offset;
            self.file.write_all(&self.current_block)?;
            self.index_entries.push((key.to_vec(), block_start));
            self.current_offset += self.current_block.len() as u64;
            self.current_block.clear();
        }

        Ok(())
    }

    pub fn finish(mut self) -> Result<()> {
        if !self.current_block.is_empty() {
            let block_start = self.current_offset;
            self.file.write_all(&self.current_block)?;
            self.index_entries.push((self.last_key, block_start));
            self.current_offset += self.current_block.len() as u64;
        }

        let index_block_offset = self.current_offset;
        for (key, offset) in &self.index_entries {
            let key_len = key.len() as u32;
            self.file.write_all(&key_len.to_le_bytes())?;
            self.file.write_all(key)?;
            self.file.write_all(&offset.to_le_bytes())?;

            self.current_offset += 4 + key.len() as u64 + 8;
        }

        let bloom_offset = self.current_offset;
        self.file.write_all(&self.bloom.encode())?;

        self.file.write_all(&index_block_offset.to_le_bytes())?;
        self.file.write_all(&bloom_offset.to_le_bytes())?;
        self.file.write_all(&SSTABLE_MAGIC.to_le_bytes())?;

        Ok(self.file.sync_all()?)
    }
}
