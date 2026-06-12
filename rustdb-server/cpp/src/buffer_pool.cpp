// Some design decisions
//  LRU-K(2) eviction
//  std::shared_mutex (readers-writer lock)
//  Free-list (std::stack over std::vector)
//  Eviction candidate selection via std::priority_queue

// Locking protocol
//   shared lock  -> fetch_page (cache hit), pinned_count
//   unique lock  -> fetch_page (cache miss / evict), unpin_page, new_page, flush_all_pages

#include "rustdb/buffer_pool.h"
#include "rustdb/frame.h"

#include <vector>
#include <stack>
#include <queue>
#include <unordered_map>
#include <array>
#include <shared_mutex>
#include <algorithm>
#include <chrono>
#include <limits>
#include <cassert>
#include <stdexcept>

namespace rustdb {

// Monotonic clock alias
using Clock     = std::chrono::steady_clock;
using Timestamp = uint64_t;

static Timestamp now_ns() noexcept {
    return static_cast<Timestamp>(
        Clock::now().time_since_epoch() / std::chrono::nanoseconds(1)
    );
}

constexpr Timestamp INF_TS = std::numeric_limits<Timestamp>::max();

// LRU-K(2) history per frame
//   Stores the last K=2 access timestamps in a fixed-size ring.
//   backward_k_dist() returns the oldest of the two — this is the eviction key.
//   Frames with fewer than K accesses get INF_TS (evict-first).
struct LruKEntry {
    static constexpr int K = 2;
    std::array<Timestamp, K> hist{};
    int                      count = 0;

    void record(Timestamp ts) noexcept {
        for (int i = K - 1; i > 0; --i) hist[i] = hist[i - 1];
        hist[0] = ts;
        count   = std::min(count + 1, K);
    }

    Timestamp backward_k_dist() const noexcept {
        return (count < K) ? INF_TS : hist[K - 1];
    }

    void reset() noexcept { hist = {}; count = 0; }
};

// Pimpl
struct BufferPool::Impl {
    // Storage
    const uint32_t              frame_count;
    std::vector<Frame>          frames;
    std::vector<LruKEntry>      lruk;

    // Lookup
    std::unordered_map<uint32_t, uint32_t> page_table;

    // Free list
    std::stack<uint32_t>        free_list;

    // Concurrency
    mutable std::shared_mutex   latch;

    // Page id counter
    uint32_t                    next_page_id = 0; // TODO: seed from disk header

    explicit Impl(uint32_t fc)
        : frame_count(fc)
        , frames(fc)
        , lruk(fc)
    {
        page_table.reserve(fc);
        for (uint32_t i = fc; i-- > 0;) free_list.push(i);
    }

    // Eviction
    // Returns the frame index of the best eviction victim, or UINT32_MAX.
    // Caller must hold unique lock.
    uint32_t pick_victim() const noexcept {
        using Entry = std::pair<Timestamp, uint32_t>;
        std::priority_queue<Entry, std::vector<Entry>, std::greater<Entry>> pq;

        for (uint32_t fi = 0; fi < frame_count; ++fi) {
            if (!frames[fi].is_evictable()) continue;
            pq.emplace(lruk[fi].backward_k_dist(), fi);
        }
        if (pq.empty()) return UINT32_MAX;
        return pq.top().second;
    }

    uint32_t evict() {
        uint32_t fi = pick_victim();
        if (fi == UINT32_MAX) return UINT32_MAX;

        Frame& f = frames[fi];
        if (f.frame_is_dirty()) {
            // TODO: DiskManager::write_page(f.frame_page_id(), f.data())
            f.set_dirty(false);
        }
        page_table.erase(f.frame_page_id());
        lruk[fi].reset();
        f.reset();
        return fi;
    }

    uint32_t alloc_frame() {
        if (!free_list.empty()) {
            uint32_t fi = free_list.top();
            free_list.pop();
            return fi;
        }
        return evict();
    }
};

// Constructor / destructor

BufferPool::BufferPool(uint32_t frame_count)
    : impl_(std::make_unique<Impl>(frame_count))
{}

BufferPool::~BufferPool() = default;

// fetch_page
//   Fast path (cache hit):  shared lock, pin, record access, return ptr.
//   Slow path (cache miss): upgrade to unique lock, alloc frame, load page.
FetchResult BufferPool::fetch_page(uint32_t page_id) {
    const Timestamp ts = now_ns();

    // Fast path: page already in pool
    {
        std::shared_lock slock(impl_->latch);
        auto it = impl_->page_table.find(page_id);
        if (it != impl_->page_table.end()) {
            uint32_t fi = it->second;
            impl_->frames[fi].pin();
            impl_->lruk[fi].record(ts);
            return { true, reinterpret_cast<uint64_t>(impl_->frames[fi].data()) };
        }
    }

    // Slow path: load page from disk into a free/evicted frame
    std::unique_lock ulock(impl_->latch);

    if (auto it = impl_->page_table.find(page_id); it != impl_->page_table.end()) {
        uint32_t fi = it->second;
        impl_->frames[fi].pin();
        impl_->lruk[fi].record(ts);
        return { true, reinterpret_cast<uint64_t>(impl_->frames[fi].data()) };
    }

    uint32_t fi = impl_->alloc_frame();
    if (fi == UINT32_MAX) return { false, 0 };

    Frame& f = impl_->frames[fi];
    // TODO: DiskManager::read_page(page_id, f.data())
    f.set_page_id(page_id);
    f.pin();
    impl_->lruk[fi].record(ts);
    impl_->page_table[page_id] = fi;

    return { true, reinterpret_cast<uint64_t>(f.data()) };
}

// unpin_page
bool BufferPool::unpin_page(uint32_t page_id, bool dirty) {
    std::unique_lock ulock(impl_->latch);

    auto it = impl_->page_table.find(page_id);
    if (it == impl_->page_table.end()) return false;

    Frame& f = impl_->frames[it->second];
    if (f.frame_pin_count() == 0) return false; // already unpinned

    if (dirty) f.set_dirty(true);
    bool now_free = f.unpin();
    
    if (now_free && !f.frame_is_dirty()) {
        impl_->page_table.erase(it);
        impl_->lruk[it->second].reset();
        f.reset();
        impl_->free_list.push(it->second);
    }
    return true;
}

// flush_all_pages
//   Uses std::for_each with a lambda — no raw loop.
void BufferPool::flush_all_pages() {
    std::unique_lock ulock(impl_->latch);

    std::for_each(
        impl_->page_table.begin(), impl_->page_table.end(),
        [&](const auto& kv) {
            Frame& f = impl_->frames[kv.second];
            if (f.frame_is_dirty()) {
                // TODO: DiskManager::write_page(kv.first, f.data())
                f.set_dirty(false);
            }
        }
    );
}

// new_page
uint32_t BufferPool::new_page() {
    std::unique_lock ulock(impl_->latch);

    uint32_t fi = impl_->alloc_frame();
    if (fi == UINT32_MAX) return UINT32_MAX;

    uint32_t new_id = impl_->next_page_id++;
    Frame& f = impl_->frames[fi];
    f.set_page_id(new_id);
    f.pin();
    impl_->lruk[fi].record(now_ns());
    impl_->page_table[new_id] = fi;
    return new_id;
}

// pinned_count - shared lock, STL count_if
uint32_t BufferPool::pinned_count() const {
    std::shared_lock slock(impl_->latch);
    return static_cast<uint32_t>(
        std::count_if(
            impl_->frames.begin(), impl_->frames.end(),
            [](const Frame& f) { return !f.is_evictable(); }
        )
    );
}

// Factory
std::unique_ptr<BufferPool> buffer_pool_new(uint32_t frame_count) {
    return std::make_unique<BufferPool>(frame_count);
}

}
