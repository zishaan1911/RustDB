# ADR 0003: WAL Design

## Status

Accepted

## Context

Write-Ahead Logging is essential for durability. We considered:
- **Circular Buffer WAL**: PostgreSQL approach
- **Log-Structured Storage**: Modern approach
- **Segmented WAL**: Hybrid approach

## Decision

Implement Segmented WAL with fuzzy checkpoints.

## Rationale

1. **Crash Recovery**: ARIES-based recovery (Analysis, Redo, Undo)
2. **Performance**: Group commit, async writes
3. **Operability**: Easy archival and point-in-time recovery
4. **Flexibility**: Segments can be independently managed
5. **Scalability**: Supports large databases

## Consequences

### Positive
- Standard ACID guarantees
- Efficient recovery
- Good operational characteristics
- PITR capability

### Negative
- Additional disk I/O overhead
- Checkpoint coordination complexity
- WAL storage requirements

## Alternatives Considered

1. **Circular Buffer**: Simpler but less operational flexibility
2. **Log-Structured**: Optimal for certain access patterns but higher complexity

## Related ADRs

- ADR 0001: Volcano Model
- ADR 0002: MVCC
