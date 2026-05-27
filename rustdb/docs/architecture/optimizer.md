# Query Optimizer

## Overview

The query optimizer transforms logical plans into efficient physical plans using various optimization techniques.

## Optimization Rules

### Predicate Pushdown
Pushes filter conditions as early as possible to reduce data processing:
- Before joins: Reduce join inputs
- Before projections: Can apply early

### Projection Pruning
Removes unnecessary columns:
- Tracks which columns are actually used
- Removes unused column projections

### Join Reordering
Finds optimal join order:
- Uses dynamic programming (Cascades optimizer style)
- Considers join selectivity and cost

### Constant Folding
Evaluates constant expressions at compile time:
- `1 + 2` becomes `3`
- Reduces runtime computation

## Cost Model

Estimates operation costs based on:
- Tuple count (cardinality)
- Data size
- I/O patterns
- CPU complexity

Supports both I/O cost and CPU cost models.

## Statistics

Maintains table statistics:
- Row counts
- Column distributions
- Index effectiveness
- Join selectivity

Statistics are updated during VACUUM operations.
