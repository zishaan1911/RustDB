// Some design notes and revisions I did compared to previous version:
//   - pin_count_ is std::atomic<uint32_t>: hot-path reads (is this frame evictable?) need no lock at all.
//   - dirty_ is std::atomic<bool> for the same reason and flush scan is lock-free.
//   - data_ is aligned to a 512-byte boundary so O_DIRECT / DMA transfers can write straight into the frame without a bounce buffer.
//   - Frame is move-constructible (vector resize) but not copyable.

#pragma once

#include <cstdint>
#include <array>
#include <atomic>
#include <memory>

namespace rustdb {

constexpr uint32_t PAGE_SIZE          = 8192;
constexpr uint32_t INVALID_PAGE_ID    = UINT32_MAX;

struct alignas(512) PageData {
    std::array<uint8_t, PAGE_SIZE> bytes{};
};

class Frame {
public:
    Frame();
    ~Frame() = default;

    Frame(const Frame&)            = delete;
    Frame& operator=(const Frame&) = delete;
    Frame(Frame&&)                 noexcept;
    Frame& operator=(Frame&&)      noexcept;

    // Lock-free Accessors

    uint32_t frame_page_id()   const noexcept { return page_id_;                        }
    bool     frame_is_dirty()  const noexcept { return dirty_.load(std::memory_order_acquire); }
    uint32_t frame_pin_count() const noexcept { return pin_count_.load(std::memory_order_acquire); }
    bool     is_evictable()    const noexcept { return pin_count_.load(std::memory_order_relaxed) == 0; }

    // Data access

    uint8_t*       data()       noexcept { return storage_->bytes.data(); }
    const uint8_t* data() const noexcept { return storage_->bytes.data(); }

    // Mutators called only by buffer pool under it's latch

    void     set_page_id(uint32_t id) noexcept { page_id_ = id; }
    void     set_dirty(bool d)        noexcept { dirty_.store(d, std::memory_order_release); }
    void     pin()                    noexcept { pin_count_.fetch_add(1, std::memory_order_acq_rel); }

    // Returns true when pin count reaches zero (frame is now evictable)
    bool     unpin() noexcept {
        uint32_t prev = pin_count_.fetch_sub(1, std::memory_order_acq_rel);
        return prev == 1; // was 1, now 0 → newly evictable
    }

    // Zero data, reset all fields. Called when evicting a clean frame.
    void reset() noexcept;

private:
    uint32_t                    page_id_   = INVALID_PAGE_ID;
    std::atomic<uint32_t>       pin_count_ = 0;
    std::atomic<bool>           dirty_     = false;
    std::unique_ptr<PageData>   storage_;  // heap-allocated so Frame is movable
};

// cxx bridge free-function accessors
uint32_t frame_page_id  (const Frame& f) noexcept;
bool     frame_is_dirty (const Frame& f) noexcept;
uint32_t frame_pin_count(const Frame& f) noexcept;

} // namespace rustdb
