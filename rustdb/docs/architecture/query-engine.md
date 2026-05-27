# Query Engine Architecture

## SQL Processing Pipeline

```
SQL Text
  ↓
Parser (converts to AST)
  ↓
Binder (resolves names)
  ↓
Type Checker (validates types)
  ↓
Logical Planner (generates logical plan)
  ↓
Optimizer (optimizes logical plan)
  ↓
Physical Planner (generates physical plan)
  ↓
Executor (runs physical plan)
  ↓
Results
```

## Parser

- Converts SQL strings to Abstract Syntax Tree (AST)
- Supports SELECT, INSERT, UPDATE, DELETE, CREATE TABLE, CREATE INDEX
- Schema definitions with ML model training

## Logical Planner

- Converts AST to logical plan (operator tree)
- Logical operators: Scan, Filter, Join, Aggregate, Sort, Projection

## Optimizer

- Predicate pushdown (filter as early as possible)
- Projection pruning (select only needed columns)
- Join reordering (find optimal join order)
- Constant folding (simplify expressions)
- Cost-based optimization

## Physical Planner

- Converts logical plan to physical operators
- Physical operators: SeqScan, IndexScan, HashJoin, MergeJoin, etc.
- Materializes plan with specific algorithms

## Executor

- Implements Volcano model (iterator interface)
- Each operator has: open(), next(), close() methods
- Row-by-row execution pipeline
