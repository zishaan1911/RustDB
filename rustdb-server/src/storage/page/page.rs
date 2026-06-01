use crate::storage::page::layout::PAGE_SIZE;
use std::fmt;

pub type PageBytes = [u8; PAGE_SIZE];

#[derive(Debug, Clone, PartialEq)]
pub enum PageError {
    Storage(String),
}

impl fmt::Display for PageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PageError::Storage(msg) => write!(f, "Storage error: {}", msg),
        }
    }
}

impl std::error::Error for PageError {}

// Represents an in-memory 8 KiB page frame managed by the buffer pool.

// It encapsulates the raw byte array transferred to and from disk storage, providing
// an interface for safety constraints and mutable data modifications.
#[derive(Clone)]
pub struct Page {
    // The actual raw memory block of 8 KiB.
    data: PageBytes,
}

impl Page {
    // Creates a zero-initialized page.
    pub fn new() -> Self {
        Self {
            data: [0u8; PAGE_SIZE],
        }
    }

    // Creates a page from an existing raw byte array.
    pub fn from_bytes(bytes: PageBytes) -> Self {
        Self { data: bytes }
    }

    // Returns an immutable reference to the underlying 8 KiB data buffer.
    pub fn data(&self) -> &PageBytes {
        &self.data
    }

    // Returns a mutable reference to the underlying 8 KiB data buffer.
    // Modifying this buffer flags the corresponding buffer frame as dirty.
    pub fn data_mut(&mut self) -> &mut PageBytes {
        &mut self.data
    }

    // Resets the page data buffer entirely to zeroes.
    pub fn clear(&mut self) {
        self.data.fill(0);
    }

    // Safely copies a slice of bytes into a specific offset range within the page.
    /// Returns `PageError::Storage` if the destination window exceeds the 8 KiB page boundaries.
    pub fn write_slice(&mut self, offset: usize, bytes: &[u8]) -> Result<(), PageError> {
        let end = offset + bytes.len();
        if end > PAGE_SIZE {
            return Err(PageError::Storage(format!(
                "Out-of-bounds page write: target end index {} exceeds PAGE_SIZE ({})",
                end, PAGE_SIZE
            )));
        }
        self.data[offset..end].copy_from_slice(bytes);
        Ok(())
    }

    // Safely extracts a view of a slice of bytes from a specific window inside the page.
    // Returns `PageError::Storage` if the requested window exceeds the 8 KiB page boundaries.
    pub fn read_slice(&self, offset: usize, length: usize) -> Result<&[u8], PageError> {
        let end = offset + length;
        if end > PAGE_SIZE {
            return Err(PageError::Storage(format!(
                "Out-of-bounds page read: target end index {} exceeds PAGE_SIZE ({})",
                end, PAGE_SIZE
            )));
        }
        Ok(&self.data[offset..end])
    }
}

impl Default for Page {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Page {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Page")
            .field("buffer_address", &(self.data.as_ptr() as usize))
            .field("size_bytes", &PAGE_SIZE)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_initialization_and_clearing() {
        let mut page = Page::new();
        assert_eq!(page.data()[0], 0);
        assert_eq!(page.data().len(), PAGE_SIZE);

        page.data_mut()[0] = 42;
        assert_eq!(page.data()[0], 42);

        page.clear();
        assert_eq!(page.data()[0], 0);
    }

    #[test]
    fn test_safe_slice_read_and_write() {
        let mut page = Page::new();
        let payload = [1, 2, 3, 4, 5];
        
        assert!(page.write_slice(100, &payload).is_ok());
        
        let read_back = page.read_slice(100, 5).unwrap();
        assert_eq!(read_back, &payload);
    }

    #[test]
    fn test_out_of_bounds_protection() {
        let mut page = Page::new();
        let overflow_payload = [0u8; 10];
        
        // Write boundary breach
        let write_res = page.write_slice(PAGE_SIZE - 5, &overflow_payload);
        assert!(write_res.is_err());

        // Read boundary breach
        let read_res = page.read_slice(PAGE_SIZE - 5, 10);
        assert!(read_res.is_err());
    }
}
