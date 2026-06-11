#include "rustdb/buffer_pool.h"
#include "rustdb/frame.h"

#include <vector>
#include <unordered_map>
#include <mutex>
#include <cassert>
#include <cstring>
#include <stdexcept>

namespace rustdb {

// ---------------------------------------------------------------------------
// Pimpl — all private state lives here, hidden from the header
// ---------------------------------------------------------------------------
struct BufferPool::Impl {
    uint32_t                         frame_count;
    std::vector<Frame>               frames;
    std::unordered_map<uint32_t, uint32_t> page_table; // page_id → frame_index
    std::mutex                       latch;
    uint32_t                         next_page_id = 0; // stub: real impl reads from disk

    explicit Impl(uint32_t fc) : frame_count(fc), frames(fc) {}
};

// ---------------------------------------------------------------------------
// Constructor / destructor
// ---------------------------------------------------------------------------

BufferPool::BufferPool(uint32_t frame_count)
    : impl_(std::make_unique<Impl>(frame_count))
{}

BufferPool::~BufferPool() = default;

// ---------------------------------------------------------------------------
// fetch_page
// TODO: implement LRU-K eviction when all frames are pinned.
//       For now this is a minimal stub: if page already in pool, pin and
//       return it; otherwise load into a free frame.
// ---------------------------------------------------------------------------
FetchResult BufferPool::fetch_page(uint32_t page_id) {
    std::lock_guard<std::mutex> lock(impl_->latch);

    // Already in pool?
    auto it = impl_->page_table.find(page_id);
    if (it != impl_->page_table.end()) {
        uint32_t fi = it->second;
        impl_->frames[fi].pin();
        return FetchResult{
            true,
            reinterpret_cast<uint64_t>(impl_->frames[fi].data())
        };
    }

    // Find a free frame (pin_count == 0, not dirty)
    for (uint32_t fi = 0; fi < impl_->frame_count; ++fi) {
        Frame& f = impl_->frames[fi];
        if (f.frame_pin_count() == 0 && !f.frame_is_dirty()) {
            // TODO: read page from DiskManager here
            f.set_page_id(page_id);
            f.pin();
            impl_->page_table[page_id] = fi;
            return FetchResult{ true, reinterpret_cast<uint64_t>(f.data()) };
        }
    }

    // Pool exhausted
    return FetchResult{ false, 0 };
}

// ---------------------------------------------------------------------------
// unpin_page
// ---------------------------------------------------------------------------
bool BufferPool::unpin_page(uint32_t page_id, bool dirty) {
    std::lock_guard<std::mutex> lock(impl_->latch);
    auto it = impl_->page_table.find(page_id);
    if (it == impl_->page_table.end()) return false;

    Frame& f = impl_->frames[it->second];
    if (dirty) f.set_dirty(true);
    return f.unpin();
}

// ---------------------------------------------------------------------------
// flush_all_pages
// ---------------------------------------------------------------------------
void BufferPool::flush_all_pages() {
    std::lock_guard<std::mutex> lock(impl_->latch);
    for (auto& [pid, fi] : impl_->page_table) {
        Frame& f = impl_->frames[fi];
        if (f.frame_is_dirty()) {
            // TODO: write f.data() to DiskManager(pid)
            f.set_dirty(false);
        }
    }
}

// ---------------------------------------------------------------------------
// new_page
// ---------------------------------------------------------------------------
uint32_t BufferPool::new_page() {
    std::lock_guard<std::mutex> lock(impl_->latch);
    for (uint32_t fi = 0; fi < impl_->frame_count; ++fi) {
        Frame& f = impl_->frames[fi];
        if (f.frame_pin_count() == 0 && !f.frame_is_dirty()) {
            uint32_t new_id = impl_->next_page_id++;
            if (f.frame_page_id() != Frame::INVALID_PAGE_ID) {
                impl_->page_table.erase(f.frame_page_id());
            }
            f.reset();
            f.set_page_id(new_id);
            f.pin();
            impl_->page_table[new_id] = fi;
            return new_id;
        }
    }
    return UINT32_MAX; // no free frame
}

// ---------------------------------------------------------------------------
// pinned_count
// ---------------------------------------------------------------------------
uint32_t BufferPool::pinned_count() const {
    std::lock_guard<std::mutex> lock(impl_->latch);
    uint32_t count = 0;
    for (const auto& f : impl_->frames) {
        if (f.frame_pin_count() > 0) ++count;
    }
    return count;
}

// ---------------------------------------------------------------------------
// Factory function (required by cxx)
// ---------------------------------------------------------------------------
std::unique_ptr<BufferPool> buffer_pool_new(uint32_t frame_count) {
    return std::make_unique<BufferPool>(frame_count);
}

}
