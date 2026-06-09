#include "rustdb/frame.h"
#include <cstring>

namespace rustdb {

Frame::Frame() = default;

void Frame::reset() {
    page_id_   = INVALID_PAGE_ID;
    pin_count_ = 0;
    dirty_     = false;
    data_.fill(0);
}

// Free-function accessors called by the cxx bridge
uint32_t frame_page_id(const Frame& f)   { return f.frame_page_id();   }
bool     frame_is_dirty(const Frame& f)  { return f.frame_is_dirty();  }
uint32_t frame_pin_count(const Frame& f) { return f.frame_pin_count(); }

} // namespace rustdb
