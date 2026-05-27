# Write-Ahead Logging (WAL)

## Overview

WAL ensures durability by logging changes before they're applied to the database. This enables crash recovery and point-in-time restore.

## WAL Records

Records are written before corresponding page changes:
- **Insert Records**: New tuple data
- **Update Records**: Before/after values
- **Delete Records**: Tuple identification
- **Abort/Commit**: Transaction boundaries
- **Checkpoint**: Recovery markers

## Checkpoint Strategy

Uses fuzzy checkpoints to minimize downtime:
- Asynchronously writes dirty pages
- Records checkpoint start/end in WAL
- Allows recovery to skip previously checkpointed pages

## Recovery Process

Three phases:
1. **Analysis**: Identify active transactions and dirty pages
2. **Redo**: Reapply all committed changes
3. **Undo**: Rollback incomplete transactions

## Performance Considerations

- WAL buffering for batched writes
- Group commit for throughput
- Async I/O where possible
- Configurable retention policies

See recovery.md for detailed recovery procedures.
