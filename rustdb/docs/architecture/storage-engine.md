# Storage Engine Architecture

## Overview

The storage engine is responsible for persisting data to disk and managing in-memory caches. It implements a layered architecture consisting of:

1. **Heap Files**: Direct row storage
2. **Page Management**: Fixed-size page abstraction
3. **Buffer Pool**: In-memory caching with LRU-K replacement
4. **Disk Manager**: Physical I/O abstraction

## Components

### Heap Files
- Stores tuples (rows) on disk
- Manages Record IDs (RIDs) for tuple identification
- Tracks tuple visibility for MVCC

### Page Organization
- Fixed 8KB pages (configurable)
- Page header for metadata
- Slot directory for variable-length tuples
- Checksum for integrity verification

### Buffer Pool
- LRU-K eviction strategy (tracks k most recent accesses)
- Dirty page tracking
- Latch-based concurrency control
- Configurable pool size

### Disk Manager
- File abstraction over raw storage
- Segment-based organization
- Sequential and random I/O optimization

## Key Features

- ACID compliance through buffer management
- Crash recovery support via page logging
- Concurrent access control
- Data compression (LZ4)
- Temporary space management for large operations

See individual component documentation for implementation details.
