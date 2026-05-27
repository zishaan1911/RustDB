# MVCC (Multi-Version Concurrency Control)

## Overview

RustDB uses MVCC to provide snapshot isolation without blocking readers. Each transaction sees a consistent snapshot of the database at its start time.

## Key Concepts

### Version Chain
- Each tuple maintains a version chain with commit timestamps
- Old versions are eventually reclaimed by vacuum

### Snapshot Isolation
- Readers never block writers
- Writers may block on conflicts
- Phantom reads are possible

### Visibility Determination
- Tuple is visible if:
  - Created by committed transaction with TxID < current snapshot
  - Not deleted, OR deleted by transaction >= snapshot

## Implementation

### Transaction Manager
- Maintains active transaction set
- Manages transaction IDs and commit timestamps
- Handles visibility checks

### Vacuum Process
- Runs periodically to reclaim old versions
- Cleans dead tuples and indexes
- Updates statistics

## Lock Management

- Optimistic locking for reads
- Pessimistic locking for writes
- Deadlock detection and resolution
