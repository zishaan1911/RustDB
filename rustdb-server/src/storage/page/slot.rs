use crate::storage::page::page::{Page, PageError};
use crate::storage::page::header::HEADER_SIZE;

// Total size of a single slot entry in bytes (2B offset + 2B length)
pub const SLOT_SIZE: usize = 4;

// Represents a single Slot entry pointing to a tuple's data window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slot {
    pub offset: u16,
    pub length: u16,
}

// Utility worker for reading and writing slot entries within an 8 KiB page.
pub struct PageSlots;

impl PageSlots {
    // Computes the exact byte offset in the page where a specific slot entry is stored.
    // Slots are packed sequentially right after the fixed page header.
    #[inline]
    pub fn slot_offset(slot_id: u16) -> usize {
        HEADER_SIZE + (slot_id as usize * SLOT_SIZE)
    }

    // Fetches a specific slot entry from the page by its ID.
    // Returns `PageError::Storage` if reading the slot breaches page boundaries.
    pub fn get_slot(page: &Page, slot_id: u16) -> Result<Slot, PageError> {
        let byte_offset = Self::slot_offset(slot_id);
        let bytes = page.read_slice(byte_offset, SLOT_SIZE)?;
        
        let offset = u16::from_le_bytes(bytes[0..2].try_into().unwrap());
        let length = u16::from_le_bytes(bytes[2..4].try_into().unwrap());
        
        Ok(Slot { offset, length })
    }

    // Writes or updates a specific slot entry in the page.
    // Returns `PageError::Storage` if writing the slot breaches page boundaries.
    pub fn set_slot(page: &mut Page, slot_id: u16, slot: Slot) -> Result<(), PageError> {
        let byte_offset = Self::slot_offset(slot_id);
        
        let mut bytes = [0u8; SLOT_SIZE];
        bytes[0..2].copy_from_slice(&slot.offset.to_le_bytes());
        bytes[2..4].copy_from_slice(&slot.length.to_le_bytes());
        
        page.write_slice(byte_offset, &bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slot_offset_calculation() {
        // Slot 0 should start right after the header (offset 22)
        assert_eq!(PageSlots::slot_offset(0), HEADER_SIZE);
        // Slot 1 should start 4 bytes later (offset 26)
        assert_eq!(PageSlots::slot_offset(1), HEADER_SIZE + SLOT_SIZE);
    }

    #[test]
    fn test_write_and_read_slots() {
        let mut page = Page::new();
        
        let sample_slot_0 = Slot { offset: 8000, length: 192 };
        let sample_slot_1 = Slot { offset: 7850, length: 150 };

        // Write slots into the page buffer
        PageSlots::set_slot(&mut page, 0, sample_slot_0).unwrap();
        PageSlots::set_slot(&mut page, 1, sample_slot_1).unwrap();

        // Read them back and verify integrity
        let read_slot_0 = PageSlots::get_slot(&page, 0).unwrap();
        let read_slot_1 = PageSlots::get_slot(&page, 1).unwrap();

        assert_eq!(read_slot_0, sample_slot_0);
        assert_eq!(read_slot_1, sample_slot_1);
    }
}
