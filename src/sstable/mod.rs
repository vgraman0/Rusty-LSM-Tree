pub mod builder;
pub mod reader;

/// SSTable data block size in bytes (4 KB).
pub const BLOCK_SIZE: usize = 4096;
pub const SSTABLE_MAGIC: u64 = 0x00E0_7AB1_E000_0000;

pub use builder::SSTableBuilder;
pub use reader::SSTableReader;
