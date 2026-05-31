use crate::constants::PAGE_SIZE;
use crate::error::RustDbError;
use crate::storage::page::page::Page;

// Total size of the fixed header in bytes.
// Layout structure:
//   [0..4]   page_id       (u32)
//   [4..8]   checksum      (u32)
//   [8..10]  slot_count    (u16)
//   [10..12] free_space_pointer (u16)
//   [12..14] flags        (u16)
//   [14..22] lsn          (u64)
//   [22..]  slot array + data area
pub const HEADER_SIZE: usize = 22;

// Explicit byte offsets for header fields
const PAGE_ID_OFFSET: usize = 0;
const CHECKSUM_OFFSET: usize = 4;
const SLOT_COUNT_OFFSET: usize = 8;
const FREE_SPACE_POINTER_OFFSET: usize = 10;
const FLAGS_OFFSET: usize = 12;
const LSN_OFFSET: usize = 14;

// A view wrapper around a `Page`'s raw byte buffer to read and write metadata.
// Instead of duplicating data structures, `PageHeader` directly reads from and writes to specific byte offsets in the underlying 8 KiB page array.
pub struct PageHeader;

impl PageHeader {
    // Initializes a brand new header on a raw page, setting default values.
    // The free space pointer initially starts at the very end of the page (`PAGE_SIZE`).
    pub fn initialize(page: &mut Page, page_id: u32) -> Result<(), RustDbError> {
        Self::set_page_id(page, page_id)?;
        Self::set_checksum(page, 0)?;
        Self::set_lsn(page, 0)?;
        Self::set_free_space_pointer(page, PAGE_SIZE as u16)?;
        Self::set_slot_count(page, 0)?;
        Self::set_flags(page, 0)?;
        Ok(())
    }

    //Getters

    pub fn get_page_id(page: &Page) -> Result<u32, RustDbError> {
        let bytes = page.read_slice(PAGE_ID_OFFSET, 4)?;
        Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
    }

    pub fn get_checksum(page: &Page) -> Result<u32, RustDbError> {
        let bytes = page.read_slice(CHECKSUM_OFFSET, 4)?;
        Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
    }

    pub fn get_slot_count(page: &Page) -> Result<u16, RustDbError> {
        let bytes = page.read_slice(SLOT_COUNT_OFFSET, 2)?;
        Ok(u16::from_le_bytes(bytes.try_into().unwrap()))
    }

    pub fn get_free_space_pointer(page: &Page) -> Result<u16, RustDbError> {
        let bytes = page.read_slice(FREE_SPACE_POINTER_OFFSET, 2)?;
        Ok(u16::from_le_bytes(bytes.try_into().unwrap()))
    }

    pub fn get_flags(page: &Page) -> Result<u16, RustDbError> {
        let bytes = page.read_slice(FLAGS_OFFSET, 2)?;
        Ok(u16::from_le_bytes(bytes.try_into().unwrap()))
    }

    pub fn get_lsn(page: &Page) -> Result<u64, RustDbError> {
        let bytes = page.read_slice(LSN_OFFSET, 8)?;
        Ok(u64::from_le_bytes(bytes.try_into().unwrap()))
    }

    //Setters

    pub fn set_page_id(page: &mut Page, page_id: u32) -> Result<(), RustDbError> {
        page.write_slice(PAGE_ID_OFFSET, &page_id.to_le_bytes())
    }

    pub fn set_checksum(page: &mut Page, checksum: u32) -> Result<(), RustDbError> {
        page.write_slice(CHECKSUM_OFFSET, &checksum.to_le_bytes())
    }

    pub fn set_slot_count(page: &mut Page, count: u16) -> Result<(), RustDbError> {
        page.write_slice(SLOT_COUNT_OFFSET, &count.to_le_bytes())
    }

    pub fn set_free_space_pointer(page: &mut Page, offset: u16) -> Result<(), RustDbError> {
        if (offset as usize) > PAGE_SIZE {
            return Err(RustDbError::Storage(format!(
                "Invalid free space pointer location: {}", offset
            )));
        }
        page.write_slice(FREE_SPACE_POINTER_OFFSET, &offset.to_le_bytes())
    }

    pub fn set_flags(page: &mut Page, flags: u16) -> Result<(), RustDbError> {
        page.write_slice(FLAGS_OFFSET, &flags.to_le_bytes())
    }

    pub fn set_lsn(page: &mut Page, lsn: u64) -> Result<(), RustDbError> {
        page.write_slice(LSN_OFFSET, &lsn.to_le_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_initialization_and_mutations() {
        let mut page = Page::new();
        
        PageHeader::initialize(&mut page, 1024).unwrap();
        
        assert_eq!(PageHeader::get_page_id(&page).unwrap(), 1024);
        assert_eq!(PageHeader::get_checksum(&page).unwrap(), 0);
        assert_eq!(PageHeader::get_lsn(&page).unwrap(), 0);
        // Initially points to the top of the canvas (8192)
        assert_eq!(PageHeader::get_free_space_pointer(&page).unwrap(), PAGE_SIZE as u16);
        assert_eq!(PageHeader::get_slot_count(&page).unwrap(), 0);
        assert_eq!(PageHeader::get_flags(&page).unwrap(), 0);

        // Update fields
        PageHeader::set_lsn(&mut page, 4242).unwrap();
        PageHeader::set_free_space_pointer(&mut page, 8000).unwrap();
        PageHeader::set_slot_count(&mut page, 5).unwrap();
        PageHeader::set_flags(&mut page, 3).unwrap();

        assert_eq!(PageHeader::get_lsn(&page).unwrap(), 4242);
        assert_eq!(PageHeader::get_free_space_pointer(&page).unwrap(), 8000);
        assert_eq!(PageHeader::get_slot_count(&page).unwrap(), 5);
        assert_eq!(PageHeader::get_flags(&page).unwrap(), 3);
    }

    #[test]
    fn test_offsets_match_layout_constants() {
        // Verify that offsets match the documented layout
        assert_eq!(PAGE_ID_OFFSET, 0);
        assert_eq!(CHECKSUM_OFFSET, 4);
        assert_eq!(SLOT_COUNT_OFFSET, 8);
        assert_eq!(FREE_SPACE_POINTER_OFFSET, 10);
        assert_eq!(FLAGS_OFFSET, 12);
        assert_eq!(LSN_OFFSET, 14);
        assert_eq!(HEADER_SIZE, 22);
    }
}
