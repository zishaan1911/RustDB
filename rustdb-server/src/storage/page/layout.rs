// Encode and decode helpers for reading and writing slot arrays within a page.
// A slot with `length == 0` is a tombstone (the tuple has been deleted).

use thiserror::Error;

// Constants

// Re-declared for standalone compilation
pub const PAGE_SIZE: usize = 8192;
pub const HEADER_SIZE: usize = 22;
pub const SLOT_SIZE: usize = 4;

pub const TOMBSTONE_LENGTH: u16 = 0;
pub const SLOT_COUNT_OFFSET: usize = 8;
pub const FREE_SPACE_PTR_OFFSET: usize = 10;

// Error type

#[derive(Debug, Error, PartialEq)]
pub enum LayoutError {
}

// Header field accessors

// Read `slot_count` from the page header.
pub fn read_slot_count(page: &[u8]) -> Result<u16, LayoutError> {
}

// Write `slot_count` into the page header.
pub fn write_slot_count(page: &mut [u8], count: u16) -> Result<(), LayoutError> {
}

// Read `free_space_pointer` from the page header.
pub fn read_free_space_ptr(page: &[u8]) -> Result<u16, LayoutError> {
}

// Write `free_space_pointer` into the page header.
pub fn write_free_space_ptr(page: &mut [u8], ptr: u16) -> Result<(), LayoutError> {
}

// Free-space calculation

// Return the number of bytes of contiguous free space between the end of the slot array and the start of the tuple data area.

pub fn free_space(page: &[u8]) -> Result<usize, LayoutError> {
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
pub fn write_slot(page: &mut [u8], index: u16, offset: u16, length: u16) -> Result<(), LayoutError> {
}

// Mark slot `index` as a tombstone by setting its length to 0.
pub fn tombstone_slot(page: &mut [u8], index: u16) -> Result<(), LayoutError> {
}

// Return `true` if slot `index` is a tombstone (length == 0).
pub fn is_tombstone(page: &[u8], index: u16) -> Result<bool, LayoutError> {
}

// Tuple allocation
pub fn allocate_tuple(page: &mut [u8], data_len: usize) -> Result<(u16, u16), LayoutError> {
}

// Tuple data access

// Return a slice of the tuple bytes for slot `index`.
// Returns an error if the slot is a tombstone or if the stored range falls outside the page.
pub fn tuple_data<'a>(page: &'a [u8], index: u16) -> Result<&'a [u8], LayoutError> {
}

// Return a mutable slice of the tuple bytes for slot `index`.
pub fn tuple_data_mut<'a>(page: &'a mut [u8], index: u16) -> Result<&'a mut [u8], LayoutError> {
    // Read metadata first (immutable borrow ends here).
}

// Page initialisation
pub fn init(page: &mut [u8]) -> Result<(), LayoutError> {
}

// Private helpers

#[inline]
fn check_size(page: &[u8]) -> Result<(), LayoutError> {
}

// Byte offset of the start of slot `index` within the page.
#[inline]
fn slot_base(index: u16) -> usize {
}

#[inline]
fn read_u16(page: &[u8], offset: usize) -> u16 {
}

#[inline]
fn write_u16(page: &mut [u8], offset: usize, value: u16) {
}

// Tests

#[cfg(test)]
mod tests {
    use super::*;


    fn blank_page() -> Vec<u8> {
    }

    fn fresh_page() -> Vec<u8> {
    }

    // init

    #[test]
    fn init_sets_slot_count_to_zero() {
    }

    #[test]
    fn init_sets_free_space_ptr_to_page_size() {
    }

    #[test]
    fn init_gives_maximum_free_space() {
    }

    // slot_count / free_space_ptr round-trips

    #[test]
    fn slot_count_round_trip() {
    }

    #[test]
    fn free_space_ptr_round_trip() {
    }

    // free_space

    #[test]
    fn free_space_decreases_after_each_slot() {
    }

    #[test]
    fn free_space_decreases_when_fsp_moves_down() {
    }

    // allocate_tuple

    #[test]
    fn allocate_tuple_increments_slot_count() {
    }

    #[test]
    fn allocate_tuple_moves_free_space_ptr() {
    }

    #[test]
    fn allocate_tuple_returns_correct_offset() {
    }

    #[test]
    fn allocate_tuple_returns_sequential_slot_indices() {
    }

    #[test]
    fn allocate_tuple_errors_when_full() {
        // Fill the page almost completely.
    }

    #[test]
    fn can_fit_returns_true_when_space_available() {
    }

    #[test]
    fn can_fit_returns_false_when_space_exhausted() {
    }

    // slot read/write

    #[test]
    fn write_then_read_slot_round_trips() {
    }

    #[test]
    fn read_slot_out_of_range_errors() {
    }

    #[test]
    fn write_slot_out_of_range_errors() {
    }

    //tombstone

    #[test]
    fn tombstone_slot_zeroes_length() {
    }

    #[test]
    fn tombstone_preserves_offset() {
    }

    #[test]
    fn is_tombstone_false_for_live_slot() {
    }

    // tuple_data

    #[test]
    fn tuple_data_returns_written_bytes() {
    }

    #[test]
    fn tuple_data_mut_allows_in_place_write() {
    }

    #[test]
    fn tuple_data_errors_on_tombstone() {
    }

    // multi-tuple round-trip

    #[test]
    fn multiple_tuples_do_not_overlap() {
    }

    #[test]
    fn free_space_after_multiple_allocations_is_consistent() {
    }

    // bad buffer

    #[test]
    fn all_entry_points_reject_wrong_size_buffer() {
    }
}
