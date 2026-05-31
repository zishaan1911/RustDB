// Encode and decode helpers for reading and writing slot arrays within a page.
// A slot with `length == 0` is a tombstone (the tuple has been deleted).
//
// SAFETY INVARIANTS:
// 1. Page buffer must always be exactly PAGE_SIZE (8192) bytes
// 2. slot_count * SLOT_SIZE + HEADER_SIZE <= free_space_ptr at all times
// 3. All tuple offsets must point within [HEADER_SIZE + slot_count*SLOT_SIZE, PAGE_SIZE)
// 4. Tuple ranges [offset, offset+length) must not overlap or exceed PAGE_SIZE
// 5. Tombstone slots have length == 0 but retain original offset for vacuum

use std::fmt;

// Constants

// Re-declared for standalone compilation
pub const PAGE_SIZE: usize = 8192;
pub const HEADER_SIZE: usize = 22;
pub const SLOT_SIZE: usize = 4;

pub const TOMBSTONE_LENGTH: u16 = 0;
pub const SLOT_COUNT_OFFSET: usize = 8;
pub const FREE_SPACE_PTR_OFFSET: usize = 10;

// Error type

#[derive(Debug, Clone, PartialEq)]
pub enum LayoutError {
    PageSizeMismatch { expected: usize, got: usize },
    SlotOutOfRange { index: u16, slot_count: u16 },
    InvalidSlotData { offset: u16, length: u16 },
    TombstoneSlot,
    NoSpace { data_len: usize },
}

impl fmt::Display for LayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LayoutError::PageSizeMismatch { expected, got } => {
                write!(f, "page size mismatch: expected {}, got {}", expected, got)
            }
            LayoutError::SlotOutOfRange { index, slot_count } => {
                write!(
                    f,
                    "slot out of range: index {} >= slot_count {}",
                    index, slot_count
                )
            }
            LayoutError::InvalidSlotData { offset, length } => {
                write!(
                    f,
                    "invalid offset/length for slot: offset={}, length={}",
                    offset, length
                )
            }
            LayoutError::TombstoneSlot => {
                write!(f, "slot is a tombstone")
            }
            LayoutError::NoSpace { data_len } => {
                write!(
                    f,
                    "no space available to allocate tuple of size {}",
                    data_len
                )
            }
        }
    }
}

impl std::error::Error for LayoutError {}

// Header field accessors

// Read `slot_count` from the page header.
pub fn read_slot_count(page: &[u8]) -> Result<u16, LayoutError> {
    check_size(page)?;
    Ok(read_u16(page, SLOT_COUNT_OFFSET))
}

// Write `slot_count` into the page header.
pub fn write_slot_count(page: &mut [u8], count: u16) -> Result<(), LayoutError> {
    check_size(page)?;
    write_u16(page, SLOT_COUNT_OFFSET, count);
    Ok(())
}

// Read `free_space_pointer` from the page header.
pub fn read_free_space_ptr(page: &[u8]) -> Result<u16, LayoutError> {
    check_size(page)?;
    Ok(read_u16(page, FREE_SPACE_PTR_OFFSET))
}

// Write `free_space_pointer` into the page header.
pub fn write_free_space_ptr(page: &mut [u8], ptr: u16) -> Result<(), LayoutError> {
    check_size(page)?;
    write_u16(page, FREE_SPACE_PTR_OFFSET, ptr);
    Ok(())
}

// Free-space calculation

// Return the number of bytes of contiguous free space between the end of the slot array and the start of the tuple data area.

pub fn free_space(page: &[u8]) -> Result<usize, LayoutError> {
    check_size(page)?;
    let slot_count = read_slot_count(page)?;
    let free_space_ptr = read_free_space_ptr(page)?;
    let slot_array_end = HEADER_SIZE + (slot_count as usize) * SLOT_SIZE;
    let available = (free_space_ptr as usize).saturating_sub(slot_array_end);
    Ok(available)
}

// Return `true` if there is room to add a tuple of `data_len` bytes plus one new slot entry.
pub fn can_fit(page: &[u8], data_len: usize) -> Result<bool, LayoutError> {
    let available = free_space(page)?;
    Ok(available >= data_len + SLOT_SIZE)
}

// Slot array read/write

// Read the slot at `index` from the page, returning `(offset, length)`.
pub fn read_slot(page: &[u8], index: u16) -> Result<(u16, u16), LayoutError> {
    check_size(page)?;
    let slot_count = read_slot_count(page)?;
    if index >= slot_count {
        return Err(LayoutError::SlotOutOfRange { index, slot_count });
    }
    let base = slot_base(index);
    let offset = read_u16(page, base);
    let length = read_u16(page, base + 2);
    Ok((offset, length))
}

// Write `(offset, length)` into slot `index`.
// The slot must already exist (i.e. `index < slot_count`).  To append a new slot use [`append_slot`].
pub fn write_slot(
    page: &mut [u8],
    index: u16,
    offset: u16,
    length: u16,
) -> Result<(), LayoutError> {
    check_size(page)?;
    let slot_count = read_slot_count(page)?;
    if index >= slot_count {
        return Err(LayoutError::SlotOutOfRange { index, slot_count });
    }
    let base = slot_base(index);
    write_u16(page, base, offset);
    write_u16(page, base + 2, length);
    Ok(())
}

// Mark slot `index` as a tombstone by setting its length to 0.
pub fn tombstone_slot(page: &mut [u8], index: u16) -> Result<(), LayoutError> {
    check_size(page)?;
    let slot_count = read_slot_count(page)?;
    if index >= slot_count {
        return Err(LayoutError::SlotOutOfRange { index, slot_count });
    }
    let base = slot_base(index);
    write_u16(page, base + 2, TOMBSTONE_LENGTH);
    Ok(())
}

// Return `true` if slot `index` is a tombstone (length == 0).
pub fn is_tombstone(page: &[u8], index: u16) -> Result<bool, LayoutError> {
    let (_offset, length) = read_slot(page, index)?;
    Ok(length == TOMBSTONE_LENGTH)
}

// Tuple allocation
pub fn allocate_tuple(page: &mut [u8], data_len: usize) -> Result<(u16, u16), LayoutError> {
    check_size(page)?;

    // Bounds check: data_len must fit in u16
    if data_len > u16::MAX as usize {
        return Err(LayoutError::NoSpace { data_len });
    }

    // Get current state to avoid redundant reads
    let slot_count = read_slot_count(page)?;
    let free_space_ptr = read_free_space_ptr(page)?;
    let free_space_ptr_usize = free_space_ptr as usize;

    // Calculate required space: slot metadata + tuple data
    let slot_array_end = HEADER_SIZE + (slot_count as usize) * SLOT_SIZE;
    let new_slot_array_end = slot_array_end + SLOT_SIZE;
    let new_offset = free_space_ptr_usize.saturating_sub(data_len);

    // SAFETY: Check invariant that slot array never overlaps with tuple data
    if new_offset < new_slot_array_end {
        return Err(LayoutError::NoSpace { data_len });
    }

    // Write the new slot at the current end of the slot array
    let base = slot_base(slot_count);
    write_u16(page, base, new_offset as u16);
    write_u16(page, base + 2, data_len as u16);

    // Update metadata: must be atomic from caller's perspective
    let new_slot_count = slot_count.saturating_add(1);
    write_slot_count(page, new_slot_count)?;
    write_free_space_ptr(page, new_offset as u16)?;

    Ok((slot_count, new_offset as u16))
}

// Tuple data access

// Return a slice of the tuple bytes for slot `index`.
// Returns an error if the slot is a tombstone or if the stored range falls outside the page.
pub fn tuple_data<'a>(page: &'a [u8], index: u16) -> Result<&'a [u8], LayoutError> {
    check_size(page)?;

    // Check if tombstone
    if is_tombstone(page, index)? {
        return Err(LayoutError::TombstoneSlot);
    }

    let (offset, length) = read_slot(page, index)?;
    let start = offset as usize;
    let end = start.saturating_add(length as usize);

    // SAFETY: Bounds check both start and end
    if start >= PAGE_SIZE || end > PAGE_SIZE {
        return Err(LayoutError::InvalidSlotData { offset, length });
    }

    Ok(&page[start..end])
}

// Return a mutable slice of the tuple bytes for slot `index`.
// SAFETY: Ensures no simultaneous mutable access to same tuple via Rust's borrow checker
pub fn tuple_data_mut<'a>(page: &'a mut [u8], index: u16) -> Result<&'a mut [u8], LayoutError> {
    // Read metadata first (immutable borrow ends after this block)
    let (offset, length) = {
        check_size(page)?;

        let slot_count = read_slot_count(page)?;
        if index >= slot_count {
            return Err(LayoutError::SlotOutOfRange { index, slot_count });
        }

        let base = slot_base(index);
        let offset = read_u16(page, base);
        let length = read_u16(page, base + 2);
        (offset, length)
    };

    if length == TOMBSTONE_LENGTH {
        return Err(LayoutError::TombstoneSlot);
    }

    let start = offset as usize;
    let end = start.saturating_add(length as usize);

    // SAFETY: Bounds check both start and end
    if start >= PAGE_SIZE || end > PAGE_SIZE {
        return Err(LayoutError::InvalidSlotData { offset, length });
    }

    Ok(&mut page[start..end])
}

// Page initialisation
pub fn init(page: &mut [u8]) -> Result<(), LayoutError> {
    check_size(page)?;
    // Initialize slot count to 0
    write_slot_count(page, 0)?;
    // Initialize free space pointer to end of page
    write_free_space_ptr(page, PAGE_SIZE as u16)?;
    Ok(())
}

// Private helpers

/// Validates that page buffer is exactly PAGE_SIZE bytes.
/// SAFETY: This is the first check on every public entry point to prevent buffer overflows.
#[inline]
fn check_size(page: &[u8]) -> Result<(), LayoutError> {
    if page.len() != PAGE_SIZE {
        return Err(LayoutError::PageSizeMismatch {
            expected: PAGE_SIZE,
            got: page.len(),
        });
    }
    Ok(())
}

/// Calculates byte offset of slot `index` within the page.
/// SAFETY: Assumes index is already validated by caller.
#[inline]
fn slot_base(index: u16) -> usize {
    HEADER_SIZE + (index as usize) * SLOT_SIZE
}

/// Reads a u16 value in little-endian format from the specified offset.
/// SAFETY: Caller must ensure offset + 2 <= page.len() (guaranteed by check_size)
#[inline]
fn read_u16(page: &[u8], offset: usize) -> u16 {
    // SAFETY: Safe because check_size ensures page.len() == PAGE_SIZE
    // All valid offsets have +2 within bounds by construction
    debug_assert!(offset + 2 <= page.len(), "read_u16 offset overflow");

    let bytes = &page[offset..offset + 2];
    u16::from_le_bytes([bytes[0], bytes[1]])
}

/// Writes a u16 value in little-endian format to the specified offset.
/// SAFETY: Caller must ensure offset + 2 <= page.len() (guaranteed by check_size)
#[inline]
fn write_u16(page: &mut [u8], offset: usize, value: u16) {
    // SAFETY: Safe because check_size ensures page.len() == PAGE_SIZE
    // All valid offsets have +2 within bounds by construction
    debug_assert!(offset + 2 <= page.len(), "write_u16 offset overflow");

    let bytes = value.to_le_bytes();
    page[offset] = bytes[0];
    page[offset + 1] = bytes[1];
}
