#pragma once

#include <cstdint>
#include <array>

namespace rustdb {

constexpr uint32_t PAGE_SIZE = 8192; // 8 KiB — must match constants.rs

class Frame {
public:
    Frame();

    // Accessors exposed to Rust via the cxx bridge

    uint32_t frame_page_id()  const { return page_id_; }
    bool     frame_is_dirty() const { return dirty_;    }
    uint32_t frame_pin_count() const { return pin_count_; }

    // ── Internal interface used only by BufferPool ────────────────────────

    // Returns raw pointer to this frame's page data (8 KiB).
    uint8_t* data() { return data_.data(); }
    const uint8_t* data() const { return data_.data(); }

    void set_page_id(uint32_t id)  { page_id_  = id;    }
    void set_dirty(bool d)         { dirty_    = d;      }
    void pin()                     { ++pin_count_;       }
    bool unpin()  {
        if (pin_count_ > 0) { --pin_count_; }
        return pin_count_ == 0;
    }
    void reset();  // zero data, clear dirty, set page_id = INVALID_PAGE_ID

    static constexpr uint32_t INVALID_PAGE_ID = UINT32_MAX;

private:
    uint32_t                  page_id_   = INVALID_PAGE_ID;
    uint32_t                  pin_count_ = 0;
    bool                      dirty_     = false;
    std::array<uint8_t, PAGE_SIZE> data_ = {};
};

// Free-function accessors — required by cxx bridge (cxx calls free functions,
// not member functions, for opaque types when the type isn't Pin<&mut T>).
uint32_t frame_page_id(const Frame& f);
bool     frame_is_dirty(const Frame& f);
uint32_t frame_pin_count(const Frame& f);

} // namespace rustdb
