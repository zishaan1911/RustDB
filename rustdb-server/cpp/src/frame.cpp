#include "rustdb/frame.h"
#include <cstring>

namespace rustdb {

Frame::Frame() : storage_(std::make_unique<PageData>()) {}

Frame::Frame(Frame&& o) noexcept
    : page_id_  (o.page_id_)
    , pin_count_(o.pin_count_.load(std::memory_order_relaxed))
    , dirty_    (o.dirty_.load(std::memory_order_relaxed))
    , storage_  (std::move(o.storage_))
{
    o.page_id_   = INVALID_PAGE_ID;
    o.pin_count_.store(0, std::memory_order_relaxed);
    o.dirty_.store(false, std::memory_order_relaxed);
}

Frame& Frame::operator=(Frame&& o) noexcept {
    if (this != &o) {
        page_id_   = o.page_id_;
        pin_count_.store(o.pin_count_.load(std::memory_order_relaxed), std::memory_order_relaxed);
        dirty_.store(o.dirty_.load(std::memory_order_relaxed),         std::memory_order_relaxed);
        storage_   = std::move(o.storage_);
        o.page_id_ = INVALID_PAGE_ID;
        o.pin_count_.store(0,     std::memory_order_relaxed);
        o.dirty_.store(false,     std::memory_order_relaxed);
    }
    return *this;
}

void Frame::reset() noexcept {
    page_id_ = INVALID_PAGE_ID;
    pin_count_.store(0,     std::memory_order_release);
    dirty_.store(false,     std::memory_order_release);
    if (storage_) storage_->bytes.fill(0);
}

// cxx bridge free functions
uint32_t frame_page_id  (const Frame& f) noexcept { return f.frame_page_id();   }
bool     frame_is_dirty (const Frame& f) noexcept { return f.frame_is_dirty();  }
uint32_t frame_pin_count(const Frame& f) noexcept { return f.frame_pin_count(); }

}
