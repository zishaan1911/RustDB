# Recovery Architecture

## Overview

RustDB uses ARIES-based recovery (Analysis, Redo, Undo) to restore database consistency after crashes.

## Crash Recovery Phases

### 1. Analysis Phase
- Scan WAL from last checkpoint
- Identify:
  - Active transactions at crash
  - Dirty pages to recover
  - Redo list and undo list

### 2. Redo Phase
- Apply all changes from active transactions
- Uses LSN (Log Sequence Number) for idempotency
- Reapplies committed and uncommitted changes

### 3. Undo Phase
- Undo changes of uncommitted transactions
- Follows undo links in WAL
- Generates CLR (Compensation Log Records)

## Checkpointing

- **Fuzzy Checkpoints**: Non-blocking checkpoint
- Tracks dirty page table
- Enables faster recovery by skipping processed WAL

## Point-in-Time Recovery

- Archive WAL segments
- Identify target LSN or timestamp
- Replay WAL up to target point

## Crash Recovery Guarantees

- **Atomicity**: All-or-nothing transactions
- **Consistency**: Valid database state
- **Isolation**: Transaction isolation preserved
- **Durability**: Committed data never lost
