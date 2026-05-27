// Buffer Pool Module
pub mod buffer_pool;
pub mod dirty_page_table;
pub mod frame;
pub mod latch;
pub mod replacer;

pub use buffer_pool::BufferPool;
pub use frame::Frame;
