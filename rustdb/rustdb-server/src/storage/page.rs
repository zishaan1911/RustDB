use crc32fast::Hasher as Crc32Hasher;
use std::mem;

use crate::error::{Result, RustDbError};

pub const PAGE_SIZE: usize = 8 * 1024;
pub const HEADER_SIZE: usize = mem::size_of::<PageHeader>();
pub const SLOT_SIZE: usize = mem::size_of::<Slot>();
pub const INVALID_PAGE_ID: u32 = u32::MAX;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PageType {
    Free = 0,
    Heap = 1,
    BtreeInternal = 2,
    BtreeLeaf = 3,
    HashBucket = 4,
    Toast = 5,
}

impl TryFrom<u8> for PageType {
    type Error = RustDbError;

    // Convert a raw byte into a page type.
    fn try_from(v: u8) -> Result<Self> {
        match v {
            0 => Ok(Self::Free),
            1 => Ok(Self::Heap),
            2 => Ok(Self::BtreeInternal),
            3 => Ok(Self::BtreeLeaf),
            4 => Ok(Self::HashBucket),
            5 => Ok(Self::Toast),
            _ => Err(RustDbError::Internal(format!("unknown page type byte: {v}"))),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct PageHeader {
    pub page_id: u32,
    pub page_type: u8,
    pub free_lower: u16,
    pub free_upper: u16,
    pub slot_count: u16,
    pub lsn: u64,
    pub checksum: u32,
    _reserved: [u8; 7],
}

const _: () = assert!(mem::size_of::<PageHeader>() == 32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C, packed)]
pub struct Slot {
    pub offset: u16,
    pub length: u16,
}

impl Slot {
    // Special marker for a deleted slot.
    pub const DEAD: Self = Self { offset: 0, length: 0 };

    // Check whether this slot points to a live tuple.
    pub fn is_live(self) -> bool {
        self.offset != 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RecordId {
    pub page_id: u32,
    pub slot_id: u16,
}

impl RecordId {
    // Create a record identifier from page and slot ids.
    pub fn new(page_id: u32, slot_id: u16) -> Self {
        Self { page_id, slot_id }
    }
}

pub struct Page {
    data: Box<[u8; PAGE_SIZE]>,
    pub dirty: bool,
    pub pin_count: u32,
}

impl Page {
    // Create an empty page with header fields initialized.
    pub fn new(page_id: u32, page_type: PageType) -> Self {
        let mut page = Self {
            data:      Box::new([0u8; PAGE_SIZE]),
            dirty:     true,
            pin_count: 0,
        };

        let hdr = page.header_mut();
        hdr.page_id = page_id;
        hdr.page_type = page_type as u8;
        hdr.free_lower = HEADER_SIZE as u16;
        hdr.free_upper = PAGE_SIZE as u16;
        hdr.slot_count = 0;
        hdr.lsn = 0;
        hdr.checksum = 0;
        hdr._reserved = [0u8; 7];

        page
    }

    // Load a page from raw bytes and verify its checksum.
    pub fn from_bytes(bytes: &[u8; PAGE_SIZE]) -> Result<Self> {
        let mut page = Self {
            data:      Box::new(*bytes),
            dirty:     false,
            pin_count: 0,
        };

        let stored = page.header().checksum;
        let computed = page.compute_checksum();
        if stored != computed {
            return Err(RustDbError::PageCorrupt {
                page_id: page.header().page_id,
            });
        }

        Ok(page)
    }

    // Return the page header from the raw page bytes.
    pub fn header(&self) -> &PageHeader {
        unsafe { &*(self.data.as_ptr() as *const PageHeader) }
    }

    // Return a mutable reference to the page header.
    fn header_mut(&mut self) -> &mut PageHeader {
        unsafe { &mut *(self.data.as_mut_ptr() as *mut PageHeader) }
    }

    // Add a tuple to the page and return its slot id.
    pub fn insert_tuple(&mut self, tuple: &[u8]) -> Option<u16> {
        let tuple_len = tuple.len() as u16;
        let needed = tuple_len + SLOT_SIZE as u16;

        if self.free_space() < needed as usize {
            return None;
        }

        let new_free_upper = self.header().free_upper - tuple_len;
        let tuple_offset = new_free_upper;
        let start = tuple_offset as usize;
        self.data[start..start + tuple.len()].copy_from_slice(tuple);

        let slot_id = self.header().slot_count;
        let slot_off = HEADER_SIZE + slot_id as usize * SLOT_SIZE;
        let slot = Slot { offset: tuple_offset, length: tuple_len };
        unsafe {
            let slot_ptr = self.data.as_mut_ptr().add(slot_off) as *mut Slot;
            slot_ptr.write_unaligned(slot);
        }

        let hdr = self.header_mut();
        hdr.free_upper = new_free_upper;
        hdr.free_lower = (slot_off + SLOT_SIZE) as u16;
        hdr.slot_count = slot_id + 1;

        self.dirty = true;
        Some(slot_id)
    }

    // Mark a tuple slot as deleted without reclaiming space.
    pub fn delete_tuple(&mut self, slot_id: u16) -> Result<()> {
        let slot_count = self.header().slot_count;
        if slot_id >= slot_count {
            return Err(RustDbError::Internal(format!(
                "slot_id {slot_id} out of range (slot_count={slot_count})"
            )));
        }

        let slot_off = HEADER_SIZE + slot_id as usize * SLOT_SIZE;
        unsafe {
            let slot_ptr = self.data.as_mut_ptr().add(slot_off) as *mut Slot;
            slot_ptr.write_unaligned(Slot::DEAD);
        }

        self.dirty = true;
        Ok(())
    }

    // Get the stored bytes for a live tuple.
    pub fn get_tuple(&self, slot_id: u16) -> Option<&[u8]> {
        let slot = self.slot(slot_id)?;
        if !slot.is_live() {
            return None;
        }
        let start = slot.offset as usize;
        let end = start + slot.length as usize;
        Some(&self.data[start..end])
    }

    // Read a slot entry from the slot directory.
    fn slot(&self, slot_id: u16) -> Option<Slot> {
        if slot_id >= self.header().slot_count {
            return None;
        }
        let off = HEADER_SIZE + slot_id as usize * SLOT_SIZE;
        let slot = unsafe {
            let ptr = self.data.as_ptr().add(off) as *const Slot;
            ptr.read_unaligned()
        };
        Some(slot)
    }

    // Return every live tuple on the page.
    pub fn tuples(&self) -> impl Iterator<Item = (u16, &[u8])> {
        (0..self.header().slot_count).filter_map(move |id| {
            self.get_tuple(id).map(|b| (id, b))
        })
    }

    // Compute how much free space is left on the page.
    pub fn free_space(&self) -> usize {
        let h = self.header();
        h.free_upper as usize - h.free_lower as usize
    }

    // Compute the page checksum while skipping the checksum field.
    fn compute_checksum(&self) -> u32 {
        const CHECKSUM_OFFSET: usize = 20;
        let mut h = Crc32Hasher::new();
        h.update(&self.data[..CHECKSUM_OFFSET]);
        h.update(&self.data[CHECKSUM_OFFSET + 4..]);
        h.finalize()
    }

    // Write the checksum into the header and return the raw page bytes.
    pub fn as_bytes_for_flush(&mut self) -> &[u8; PAGE_SIZE] {
        let checksum = self.compute_checksum();
        self.header_mut().checksum = checksum;
        self.dirty = false;
        &*self.data
    }

    // Return the raw page bytes without updating the checksum.
    pub fn raw_bytes(&self) -> &[u8; PAGE_SIZE] {
        &*self.data
    }
}

impl std::fmt::Debug for Page {
    // Format the page metadata for debug output.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let h = self.header();
        f.debug_struct("Page")
            .field("page_id", &h.page_id)
            .field("page_type", &h.page_type)
            .field("slot_count", &h.slot_count)
            .field("free_space", &self.free_space())
            .field("lsn", &h.lsn)
            .field("dirty", &self.dirty)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Create a heap page for tests.
    fn make_heap_page(id: u32) -> Page {
        Page::new(id, PageType::Heap)
    }

    #[test]
    fn page_size_is_8kib() {
        assert_eq!(PAGE_SIZE, 8192);
    }

    #[test]
    fn header_is_32_bytes() {
        assert_eq!(HEADER_SIZE, 32);
    }

    #[test]
    fn new_page_full_free_space() {
        let p = make_heap_page(1);
        assert_eq!(p.free_space(), PAGE_SIZE - HEADER_SIZE);
    }

    #[test]
    fn insert_and_retrieve_tuple() {
        let mut p = make_heap_page(1);
        let data = b"hello rustdb";
        let sid = p.insert_tuple(data).expect("should insert");
        let got = p.get_tuple(sid).expect("should retrieve");
        assert_eq!(got, data);
    }

    #[test]
    fn insert_multiple_tuples() {
        let mut p = make_heap_page(1);
        for i in 0u8..10 {
            let d = vec![i; 20];
            p.insert_tuple(&d).unwrap();
        }
        assert_eq!(p.header().slot_count, 10);
        assert_eq!(p.tuples().count(), 10);
    }

    #[test]
    fn delete_tuple_returns_none() {
        let mut p = make_heap_page(1);
        let sid = p.insert_tuple(b"to be deleted").unwrap();
        p.delete_tuple(sid).unwrap();
        assert!(p.get_tuple(sid).is_none());
    }

    #[test]
    fn insert_returns_none_when_full() {
        let mut p = make_heap_page(1);
        let big = vec![0u8; 4000];
        p.insert_tuple(&big).unwrap();
        let result = p.insert_tuple(&big);
        assert!(result.is_none());
    }

    #[test]
    fn checksum_roundtrip() {
        let mut p = make_heap_page(7);
        p.insert_tuple(b"checksum test").unwrap();
        let bytes = *p.as_bytes_for_flush();
        let p2 = Page::from_bytes(&bytes).expect("valid checksum");
        assert_eq!(p2.header().page_id, 7);
    }

    #[test]
    fn corrupted_page_rejected() {
        let mut p = make_heap_page(3);
        let mut bytes = *p.as_bytes_for_flush();
        bytes[100] ^= 0xFF;
        let result = Page::from_bytes(&bytes);
        assert!(matches!(result, Err(RustDbError::PageCorrupt { .. })));
    }
}
