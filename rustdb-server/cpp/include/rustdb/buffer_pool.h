#pragma once

#include <cstdint>
#include <memory>
#include "frame.h"

namespace rustdb {

struct PageId {
    uint32_t page_id;
    uint16_t slot_id;
};

struct FetchResult {
    bool     success;
    uint64_t data_ptr; // cast to uint8_t* inside fetch_page implementation
};

// ---------------------------------------------------------------------------
// BufferPool
// ---------------------------------------------------------------------------

class BufferPool {
public:
    explicit BufferPool(uint32_t frame_count);
    ~BufferPool();

    // Pin page into memory; caller must call unpin_page when done.
    FetchResult fetch_page(uint32_t page_id);

    // Release pin. dirty=true schedules page for flush.
    bool unpin_page(uint32_t page_id, bool dirty);

    // Flush all dirty pages to disk (checkpoint / shutdown).
    void flush_all_pages();

    // Allocate a new page. Returns UINT32_MAX if no free frames.
    uint32_t new_page();

    // Diagnostic: number of currently pinned frames.
    uint32_t pinned_count() const;

private:
    struct Impl;
    std::unique_ptr<Impl> impl_; // pimpl — keeps internals out of the header
};
std::unique_ptr<BufferPool> buffer_pool_new(uint32_t frame_count);

}
