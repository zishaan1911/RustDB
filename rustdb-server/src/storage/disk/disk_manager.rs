//! Low-level page I/O.
//!
//! `DiskManager` owns a single open file and exposes exactly two operations:
//! read a page and write a page.  It knows nothing about tables, OIDs, or the
//! buffer pool — those concerns belong to `file_manager.rs` and
//! `buffer_pool.rs` respectively.

use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
    path::Path,
};

use thiserror::Error;

// Must match `constants::PAGE_SIZE`.
pub const PAGE_SIZE: usize = 8192;

/// Opaque page identifier.  Page 0 is the first page of the file.
pub type PageId = u32;

// Error

#[derive(Debug, Error)]
pub enum DiskManagerError {
    #[error("I/O error on page {page_id}: {source}")]
    Io {
        page_id: PageId,
        #[source]
        source: io::Error,
    },

    #[error("I/O error during sync: {0}")]
    SyncError(#[source] io::Error),

    #[error("I/O error opening file: {0}")]
    OpenError(#[source] io::Error),

    #[error(
        "short read on page {page_id}: expected {expected} bytes, got {actual}"
    )]
    ShortRead {
        page_id: PageId,
        expected: usize,
        actual: usize,
    },

    #[error("buffer length {actual} must be exactly {expected} (PAGE_SIZE)")]
    BadBufferLength { actual: usize, expected: usize },
}

// DiskManager

// Reads and writes 8 KiB pages to/from a single file.
#[derive(Debug)]
pub struct DiskManager {
    file: File,
    // Cached file size in bytes, kept in sync with every write so we can compute `num_pages` without an extra syscall.
    file_size: u64,
}

impl DiskManager {
    // Open an existing file or create a new one at `path`.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, DiskManagerError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .map_err(DiskManagerError::OpenError)?;

        let file_size = file
            .metadata()
            .map_err(DiskManagerError::OpenError)?
            .len();

        Ok(Self { file, file_size })
    }

    // A freshly created file returns 0.
    pub fn num_pages(&self) -> u32 {
        (self.file_size / PAGE_SIZE as u64) as u32
    }

    // Read page `page_id` into `buf`.
    /// `buf` must be exactly `PAGE_SIZE` bytes.  Returns [`DiskManagerError::ShortRead`] if the file is shorter than expected.
    pub fn read_page(
        &mut self,
        page_id: PageId,
        buf: &mut [u8],
    ) -> Result<(), DiskManagerError> {
        self.check_buf(page_id, buf)?;

        let offset = page_offset(page_id);
        self.file
            .seek(SeekFrom::Start(offset))
            .map_err(|e| DiskManagerError::Io { page_id, source: e })?;

        let n = self
            .file
            .read(buf)
            .map_err(|e| DiskManagerError::Io { page_id, source: e })?;

        if n != PAGE_SIZE {
            return Err(DiskManagerError::ShortRead {
                page_id,
                expected: PAGE_SIZE,
                actual: n,
            });
        }

        Ok(())
    }

    // If `page_id` is beyond the current end of file the file is extended (the gap is zero-filled by the OS). 
    // This does **not** call `fsync`; use [`DiskManager::sync`] when a durability barrier is needed.
    pub fn write_page(
        &mut self,
        page_id: PageId,
        buf: &[u8],
    ) -> Result<(), DiskManagerError> {
        self.check_buf(page_id, buf)?;

        let offset = page_offset(page_id);
        self.file
            .seek(SeekFrom::Start(offset))
            .map_err(|e| DiskManagerError::Io { page_id, source: e })?;

        self.file
            .write_all(buf)
            .map_err(|e| DiskManagerError::Io { page_id, source: e })?;

        // Keep the cached file size current.
        let new_end = offset + PAGE_SIZE as u64;
        if new_end > self.file_size {
            self.file_size = new_end;
        }

        Ok(())
    }

    // `fsync` the underlying file, flushing OS page-cache to durable storage.
    // Call this after flushing dirty pages during a checkpoint.
    pub fn sync(&self) -> Result<(), DiskManagerError> {
        self.file.sync_all().map_err(DiskManagerError::SyncError)
    }

    // Allocate the next page ID without writing anything.
    // The caller is expected to call `write_page` with this ID before the page is pinned in the buffer pool.
    pub fn allocate_page(&mut self) -> PageId {
        let id = self.num_pages();
        // Extend the cached size so subsequent calls return distinct IDs even before the first write.
        self.file_size += PAGE_SIZE as u64;
        id
    }

    // private

    #[inline]
    fn check_buf(&self, page_id: PageId, buf: &[u8]) -> Result<(), DiskManagerError> {
        if buf.len() != PAGE_SIZE {
            return Err(DiskManagerError::BadBufferLength {
                actual: buf.len(),
                expected: PAGE_SIZE,
            });
        }
        let _ = page_id; // reserved for future range checks
        Ok(())
    }
}

// Byte offset of `page_id` within the file.
#[inline]
fn page_offset(page_id: PageId) -> u64 {
    page_id as u64 * PAGE_SIZE as u64
}

// Tests

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn temp_dm() -> (DiskManager, NamedTempFile) {
        let f = NamedTempFile::new().unwrap();
        let dm = DiskManager::open(f.path()).unwrap();
        (dm, f)
    }

    fn page_buf(byte: u8) -> Vec<u8> {
        vec![byte; PAGE_SIZE]
    }

    // open / num_pages

    #[test]
    fn fresh_file_has_zero_pages() {
        let (dm, _f) = temp_dm();
        assert_eq!(dm.num_pages(), 0);
    }

    #[test]
    fn existing_file_counts_pages_correctly() {
        let f = NamedTempFile::new().unwrap();
        // Pre-fill with 3 pages of zeroes.
        {
            let mut file = f.reopen().unwrap();
            file.write_all(&vec![0u8; PAGE_SIZE * 3]).unwrap();
        }
        let dm = DiskManager::open(f.path()).unwrap();
        assert_eq!(dm.num_pages(), 3);
    }

    // write_page / read_page

    #[test]
    fn write_then_read_round_trips() {
        let (mut dm, _f) = temp_dm();
        let written = page_buf(0xAB);
        dm.write_page(0, &written).unwrap();

        let mut read = vec![0u8; PAGE_SIZE];
        dm.read_page(0, &mut read).unwrap();
        assert_eq!(written, read);
    }

    #[test]
    fn write_multiple_pages_independently() {
        let (mut dm, _f) = temp_dm();

        for i in 0u8..4 {
            dm.write_page(i as PageId, &page_buf(i * 10)).unwrap();
        }

        for i in 0u8..4 {
            let mut buf = vec![0u8; PAGE_SIZE];
            dm.read_page(i as PageId, &mut buf).unwrap();
            assert!(buf.iter().all(|&b| b == i * 10));
        }
    }

    #[test]
    fn write_updates_num_pages() {
        let (mut dm, _f) = temp_dm();
        assert_eq!(dm.num_pages(), 0);
        dm.write_page(0, &page_buf(1)).unwrap();
        assert_eq!(dm.num_pages(), 1);
        dm.write_page(1, &page_buf(2)).unwrap();
        assert_eq!(dm.num_pages(), 2);
    }

    #[test]
    fn overwrite_page_preserves_neighbors() {
        let (mut dm, _f) = temp_dm();
        dm.write_page(0, &page_buf(0xAA)).unwrap();
        dm.write_page(1, &page_buf(0xBB)).unwrap();
        dm.write_page(2, &page_buf(0xCC)).unwrap();

        // Overwrite page 1.
        dm.write_page(1, &page_buf(0x11)).unwrap();

        let mut buf = vec![0u8; PAGE_SIZE];

        dm.read_page(0, &mut buf).unwrap();
        assert!(buf.iter().all(|&b| b == 0xAA));

        dm.read_page(1, &mut buf).unwrap();
        assert!(buf.iter().all(|&b| b == 0x11));

        dm.read_page(2, &mut buf).unwrap();
        assert!(buf.iter().all(|&b| b == 0xCC));
    }

    // allocate_page

    #[test]
    fn allocate_page_returns_sequential_ids() {
        let (mut dm, _f) = temp_dm();
        let a = dm.allocate_page();
        let b = dm.allocate_page();
        let c = dm.allocate_page();
        assert_eq!((a, b, c), (0, 1, 2));
    }

    #[test]
    fn allocate_page_increments_num_pages() {
        let (mut dm, _f) = temp_dm();
        dm.allocate_page();
        dm.allocate_page();
        assert_eq!(dm.num_pages(), 2);
    }

    // error cases

    #[test]
    fn read_rejects_wrong_size_buffer() {
        let (mut dm, _f) = temp_dm();
        let mut short = vec![0u8; PAGE_SIZE - 1];
        let err = dm.read_page(0, &mut short).unwrap_err();
        assert!(matches!(err, DiskManagerError::BadBufferLength { .. }));
    }

    #[test]
    fn write_rejects_wrong_size_buffer() {
        let (mut dm, _f) = temp_dm();
        let short = vec![0u8; PAGE_SIZE + 1];
        let err = dm.write_page(0, &short).unwrap_err();
        assert!(matches!(err, DiskManagerError::BadBufferLength { .. }));
    }

    #[test]
    fn read_empty_file_returns_short_read_error() {
        let (mut dm, _f) = temp_dm();
        let mut buf = vec![0u8; PAGE_SIZE];
        let err = dm.read_page(0, &mut buf).unwrap_err();
        assert!(matches!(err, DiskManagerError::ShortRead { .. }));
    }

    // sync

    #[test]
    fn sync_does_not_error_on_open_file() {
        let (mut dm, _f) = temp_dm();
        dm.write_page(0, &page_buf(0xFF)).unwrap();
        assert!(dm.sync().is_ok());
    }
}
