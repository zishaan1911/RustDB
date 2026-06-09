use std::pin::Pin;
use cxx::UniquePtr;

use crate::bridge::buffer_pool::ffi;
use crate::error::RustDbError;

// ---------------------------------------------------------------------------
// Public-facing wrapper struct
// ---------------------------------------------------------------------------

pub struct BufferPoolHandle {
    inner: UniquePtr<ffi::BufferPool>,
}

unsafe impl Send for BufferPoolHandle {}
unsafe impl Sync for BufferPoolHandle {}

impl BufferPoolHandle {
    /// Create a new buffer pool.
    ///
    /// `frame_count` is the number of 8 KiB pages to hold in memory.
    /// Default from config is 4096 (= 32 MiB). Call once at startup.
    pub fn new(frame_count: u32) -> Self {
        Self {
            inner: ffi::buffer_pool_new(frame_count),
        }
    }

    pub fn fetch_page(&mut self, page_id: u32) -> Result<PageGuard<'_>, RustDbError> {
        let result = self.inner.pin_mut().fetch_page(page_id);
        if !result.success {
            return Err(RustDbError::Storage(format!(
                "buffer pool: fetch_page({page_id}) failed — pool exhausted or I/O error"
            )));
        }
        // SAFETY: data_ptr is valid for the lifetime of this guard because
        // the page is pinned (pin_count > 0) and cannot be evicted.
        let data = unsafe {
            std::slice::from_raw_parts_mut(result.data_ptr as *mut u8, crate::constants::PAGE_SIZE)
        };
        Ok(PageGuard {
            pool: self,
            page_id,
            data,
            dirty: false,
        })
    }

    /// Allocate a new page and return its id.
    pub fn new_page(&mut self) -> Result<u32, RustDbError> {
        let id = self.inner.pin_mut().new_page();
        if id == u32::MAX {
            return Err(RustDbError::Storage("buffer pool: no free frames for new_page".into()));
        }
        Ok(id)
    }

    /// Flush all dirty pages to disk. Called at checkpoint and shutdown.
    pub fn flush_all(&mut self) {
        self.inner.pin_mut().flush_all_pages();
    }

    /// Number of currently pinned frames (for metrics / assertions).
    pub fn pinned_count(&self) -> u32 {
        self.inner.pinned_count()
    }

    // Internal: called by PageGuard::drop
    fn unpin(&mut self, page_id: u32, dirty: bool) {
        self.inner.pin_mut().unpin_page(page_id, dirty);
    }
}

// ---------------------------------------------------------------------------
// PageGuard — scoped pin on a single page
// ---------------------------------------------------------------------------

pub struct PageGuard<'pool> {
    pool:    &'pool mut BufferPoolHandle,
    page_id: u32,
    data:    &'pool mut [u8],
    dirty:   bool,
}

impl<'pool> PageGuard<'pool> {
    pub fn page_id(&self) -> u32 {
        self.page_id
    }

    /// Read-only view of the 8 KiB page bytes.
    pub fn data(&self) -> &[u8] {
        self.data
    }

    /// Mutable view. You MUST call mark_dirty() after writing, otherwise
    /// the changes may not be flushed before the page is evicted.
    pub fn data_mut(&mut self) -> &mut [u8] {
        self.data
    }

    /// Mark this page as modified. The pool will flush it at checkpoint.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}

impl Drop for PageGuard<'_> {
    fn drop(&mut self) {
        self.pool.unpin(self.page_id, self.dirty);
    }
}
