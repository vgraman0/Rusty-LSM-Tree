# RustyLSM: LSM Key-Value Store in Rust

## 1. Project Overview
The goal is to implement a persistent, file-based Key-Value (KV) store using a **Log-Structured Merge-tree (LSM)** architecture. The engine must support atomic writes, point lookups, and range scans while ensuring data integrity across process restarts.

## 2. Functional Requirements

Your implementation must provide a library API with the following signature:

### Core API

* **`open(path: PathBuf) -> Result<DB>`**: Opens a database at the specified directory. If it doesn't exist, initialize it.
* **`put(key: &[u8], value: &[u8]) -> Result<()>`**: Inserts or updates a key.
* **`get(key: &[u8]) -> Result<Option<Vec<u8>>>`**: Retrieves the latest value for a key.
* **`delete(key: &[u8]) -> Result<()>`**: Deletes a key (by inserting a "tombstone" record).
* **`scan(start: &[u8], end: &[u8]) -> Iterator`**: Returns an iterator over all keys in the given range (inclusive).

### Persistence & Durability

* **WAL (Write-Ahead Log):** Every `put` must be appended to a WAL file and `fsync`'d before the operation returns success.
* **MemTable Flush:** When the in-memory MemTable exceeds $4$ MB, it must be flushed to disk as a new **SSTable (Sorted String Table)**.
* **Crash Recovery:** Upon calling `open()`, the engine must detect any existing WAL files, replay the operations into the MemTable, and ensure no data was lost from the last successful write.

---

## 3. Storage Format Requirements (The Disk)

To ensure your database is "real" and interoperable, you must follow this file format:

### SSTable Structure (Level 0)

Each `.sst` file must be immutable and consist of:

1. **Data Blocks:** $4$ KB blocks containing sorted `(KeySize, Key, ValueSize, Value)` entries.
2. **Index Block:** A footer containing the last key of each Data Block and its byte offset within the file (for binary search).
3. **Bloom Filter:** A bitset stored in the footer to identify if a key is definitely *not* in this file.

### Manifest File

A single `MANIFEST` file must track the "Active Set" of SSTables.

* When a new SSTable is created, the Manifest is updated.
* The Manifest update must be **atomic** (e.g., write to a temp file and `rename` over the old one).

---

## 4. Technical Constraints (The "Rust" Way)

* **Zero-Copy Reading:** Use the `memmap2` crate to memory-map SSTables for fast reads.
* **Concurrency:** The `DB` handle must be thread-safe (`Send + Sync`). Use a `RwLock` or `Mutex` to protect the MemTable, but ensure the WAL write happens outside the lock to minimize contention.
* **Error Handling:** Define a custom `Error` enum using `thiserror` (e.g., `IOError`, `CorruptionError`, `SerializationError`).

---

## 5. Implementation Milestones

### Milestone 1: The In-Memory Store
Implement a `SkipList` or `BTreeMap` MemTable with the `put/get` API. No persistence yet.

| File | Struct / Trait | Key Methods |
|------|---------------|-------------|
| `src/memtable.rs` | `MemTable` (trait) | `put`, `get`, `delete`, `len`, `is_empty` |
| `src/memtable.rs` | `SkipListMemTable` | `new`, `find_update_path`, `random_level` |
| `src/error.rs` | `Error` enum | custom error types via `thiserror` |

### Milestone 2: The WAL & Recovery
Implement the WAL. Close the program, restart it, and prove the MemTable is rebuilt from the log.

| File | Struct / Trait | Key Methods |
|------|---------------|-------------|
| `src/wal.rs` | `Wal` | `create`, `open`, `append`, `recover`, `sync` |
| `src/db.rs` | `DB` | `open` (replay WAL → MemTable), `put`, `delete` (WAL + MemTable writes) |

### Milestone 3: SSTable Generation
Implement the logic to "Freeze" a MemTable and write it to a sorted `.sst` file with a basic index.

| File | Struct / Trait | Key Methods |
|------|---------------|-------------|
| `src/sstable/builder.rs` | `SSTableBuilder` | `new`, `add`, `finish` |
| `src/bloom.rs` | `BloomFilter` | `new`, `insert`, `may_contain`, `encode`, `decode` |
| `src/manifest.rs` | `Manifest` | `create`, `load`, `add_sstable`, `active_sstables` |
| `src/db.rs` | `DB` | flush logic (freeze MemTable → SSTableBuilder → new WAL) |

### Milestone 4: Point Lookups & Range Scans
Update `get()` to check the MemTable first, then the most recent SSTables in order.

| File | Struct / Trait | Key Methods |
|------|---------------|-------------|
| `src/sstable/reader.rs` | `SSTableReader` | `open`, `get`, `scan`, `may_contain` |
| `src/db.rs` | `DB` | `get` (MemTable → SSTables), `scan` (merge across all sources) |

### Milestone 5 (The "A" Grade): Compaction
Implement a background thread that merges two Level-0 SSTables into a single Level-1 SSTable, removing old versions of keys.

| File | Struct / Trait | Key Methods |
|------|---------------|-------------|
| `src/compaction.rs` | `CompactionManager` | `new`, `start`, `stop` |
| `src/manifest.rs` | `Manifest` | `remove_sstable` (cleanup after merge) |
| `src/db.rs` | `DB` | integrate compaction lifecycle (start on open, stop on drop) |
