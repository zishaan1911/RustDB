# RustDB Architecture Overview

## Introduction

RustDB is a high-performance, embeddable SQL database written in Rust, designed for modern applications with advanced features like MVCC, ML capabilities, and robust security.

## Core Components

### 1. API Layer
- HTTP REST API for client interactions
- Authentication and authorization middleware
- Request/response handling and validation

### 2. SQL Engine
- **Parser**: Converts SQL strings to AST
- **Planner**: Generates logical and physical execution plans
- **Optimizer**: Applies query optimization rules
- **Executor**: Executes plans using the Volcano model

### 3. Storage Engine
- **Buffer Pool**: Manages in-memory page cache with LRU-K replacement
- **Disk Manager**: Handles physical I/O operations
- **Heap Files**: Stores row data with MVCC visibility information
- **B-Tree Index**: For efficient data lookups
- **Hash Index**: For equality predicates

### 4. Transaction Manager
- MVCC-based isolation without blocking reads
- Lock manager for write conflicts
- Snapshot isolation by default

### 5. Write-Ahead Logging (WAL)
- Durability guarantees
- Fuzzy checkpoints for crash recovery
- Log compaction and archival

### 6. ML Engine
- Model training and inference
- Integrated ML algorithms
- Model caching and optimization

## Execution Flow

```
Client Request
    ↓
API Layer (Authentication/Authorization)
    ↓
SQL Parser (AST Generation)
    ↓
Binder & Type Checker
    ↓
Logical Planner
    ↓
Optimizer (Predicate Pushdown, Join Reordering, etc.)
    ↓
Physical Planner
    ↓
Executor (Volcano Model - next/open/close)
    ↓
Storage Engine (Buffer Pool, B-Tree, etc.)
    ↓
Response
```

## Key Design Decisions

1. **Volcano Model**: Iterator-based execution for flexibility and pipelining
2. **MVCC**: Non-blocking reads with multi-version consistency
3. **Fuzzy Checkpoints**: Reduce recovery time and I/O overhead
4. **Embedded Design**: Can be used as a library or standalone server

## Security Features

- API key authentication
- Role-based access control (RBAC)
- AES-GCM encryption for sensitive data
- Audit logging for compliance
- TLS support for client connections

## Performance Features

- LRU-K buffer pool replacement strategy
- Query optimization with cost-based planner
- Index-aware query execution
- Asynchronous I/O where possible
- Metrics and slow query logging

See individual component documentation for deeper technical details.
