pub mod checksum;
pub mod header;
pub mod layout;
pub mod page;
pub mod slot;

// Convenience re-exports
// Code outside this module can write `use crate::storage::page::Page` instead of `use crate::storage::page::page::Page`.

pub use checksum::{ChecksumError, compute as compute_checksum, verify as verify_checksum, write as write_checksum};
pub use header::{PageHeader, PAGE_HEADER_SIZE};
pub use layout::{
    LayoutError,
    PAGE_SIZE,
    HEADER_SIZE,
    SLOT_SIZE,
    TOMBSTONE_LENGTH,
    allocate_tuple,
    can_fit,
    free_space,
    init as init_layout,
    is_tombstone,
    read_free_space_ptr,
    read_slot,
    read_slot_count,
    tombstone_slot,
    tuple_data,
    tuple_data_mut,
    write_free_space_ptr,
    write_slot,
    write_slot_count,
};
pub use page::Page;
pub use slot::Slot;
