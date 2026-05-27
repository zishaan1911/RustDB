# ADR 0001: Volcano Model for Query Execution

## Status

Accepted

## Context

We needed to choose an execution model for the query executor. The main candidates were:
- **Volcano Model (Iterator Model)**: Pull-based execution with open/next/close interface
- **Compilation Model**: Code generation for each query
- **Vectorized Model**: Batch processing of tuples

## Decision

We chose the **Volcano Model** as the primary execution model.

## Rationale

1. **Simplicity**: Easier to understand and implement compared to code generation
2. **Flexibility**: Works well with dynamic SQL and complex query plans
3. **Memory Efficiency**: Pipelined execution doesn't require materializing intermediate results
4. **Operator Composition**: Easy to add new operators and query patterns
5. **Proven Track Record**: Used successfully in PostgreSQL, MySQL, and other major databases

## Consequences

### Positive
- Straightforward implementation for most operators
- Easy to debug and profile
- Good support for various query patterns
- Minimal memory overhead

### Negative
- Function call overhead compared to compiled code
- May not achieve peak throughput compared to vectorized approaches
- Requires careful memory management for large result sets

## Alternatives Considered

1. **Compilation Model**: Would provide better performance but significantly higher complexity
2. **Vectorized Model**: Would provide better cache locality but is more complex to implement

## Related ADRs

- ADR 0002: MVCC Isolation
- ADR 0003: WAL Design
