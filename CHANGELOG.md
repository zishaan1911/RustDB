_Released on 2026/06/11 21:40_

_Bug Fixed_
- Fixed compiler warning regarding unused mutable variable `page` in `rustdb-server/src/storage/page/checksum.rs`.

_Released on 2026/06/12 10:00_

_Enhancements_

- Refactored Frame representation and accessors (rustdb-server/cpp/include/rustdb/frame.h, rustdb-server/cpp/src/frame.cpp):
PageData is now aligned to 512 bytes to support O_DIRECT/DMA-friendly IO and is heap-allocated.
Frame is move-constructible and move-assignable but not copyable to allow vector resizing without accidental copies.
pin_count_ and dirty_ are implemented as atomics; lock-free accessors were added for page id, dirty flag, and pin count.
Implemented move constructor and move-assignment operator; added reset() which zeroes storage and resets metadata.

_New feature_

- Implemented BufferPool (rustdb-server/cpp/include/rustdb/buffer_pool.h, rustdb-server/cpp/src/buffer_pool.cpp):
Public API: fetch_page, unpin_page, flush_all_pages, new_page, pinned_count.
FetchResult struct returns success and a data_ptr (uint64_t) that callers should cast to a pointer.
Internal design: LRU-K(2) eviction history, free-list for free frames, page_table for lookup, and std::shared_mutex for concurrent access. Eviction selection implemented via a priority queue using backward-K distance as the key.
