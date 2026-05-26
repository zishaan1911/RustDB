# RustDB - Full Software Requirements Specification (Individual phases to be expanded later)

**Version:** 1.0.1  
**Status:** Draft  
**Language:** Rust (edition 2021)  
**Deployment Target:** Single-node server (cloud-ready)  
**Last Updated:** 2026-05-24  

---

## Table of Contents

1. [Project Overview](#1-project-overview)
2. [Phase 1 - Backend Core](#2-phase-1--backend-core)
3. [Phase 2 - Networking Layer](#3-phase-2--networking-layer)
4. [Phase 3 - Security](#4-phase-3--security)
5. [Phase 4 - Frontend & Client Tools](#5-phase-4--frontend--client-tools)
6. [Phase 5 - ML Engine](#6-phase-5--ml-engine)
7. [Phase 6 - Optimization & Performance](#7-phase-6--optimization--performance)
8. [Phase 7 - Observability & Monitoring](#8-phase-7--observability--monitoring)
9. [Phase 8 - Cloud & Infrastructure](#9-phase-8--cloud--infrastructure)
10. [Phase 9 - CI/CD Pipeline](#10-phase-9--cicd-pipeline)
11. [Phase 10 - Testing Strategy](#11-phase-10--testing-strategy)
12. [Non-Functional Requirements](#12-non-functional-requirements)
13. [Data Model Reference](#13-data-model-reference)
14. [Module & Crate Structure](#14-module--crate-structure)
15. [Dependency Manifest](#15-dependency-manifest)
16. [Glossary](#16-glossary)

---

## 1. Project Overview

RustDB is a production-grade, single-node relational database management system (RDBMS) written in Rust. It provides ACID-compliant relational storage, a Core SQL query engine, native machine-learning training and inference via SQL syntax, and an HTTP/REST API surface — all in a single self-contained binary.

### Design Pillars

| Pillar | Decision |
|--------|----------|
| Language | Rust 2021 edition — memory safety without GC |
| Storage format | B+ tree primary index, heap files (8 KiB pages) |
| Concurrency | MVCC with Snapshot Isolation |
| Durability | Write-Ahead Logging (ARIES-style recovery) |
| Query surface | Core SQL + custom `TRAIN MODEL` / `PREDICT()` |
| Client protocol | HTTP/REST (JSON) over TCP |
| ML runtime | `linfa` crate — no external ML runtime dependency |

### Out of Scope (v1.0)

- Distributed / multi-node replication
- Full SQL:2023 (window functions, CTEs → v1.1)
- OAuth / OIDC authentication (→ v1.2)
- External model import — ONNX, PyTorch (→ v1.2)
- Columnar / OLAP storage engine

---

## 2. Phase 1 — Backend Core

### 2.1 SQL Parser

| ID | Requirement |
|----|-------------|
| BE-01 | Parse SQL using `sqlparser-rs`; produce a typed AST |
| BE-02 | Support `SELECT`, `INSERT`, `UPDATE`, `DELETE`, `CREATE/DROP TABLE`, `CREATE/DROP INDEX`, `BEGIN`, `COMMIT`, `ROLLBACK` |
| BE-03 | Support `INNER`, `LEFT`, `RIGHT`, `CROSS` joins |
| BE-04 | Support `GROUP BY` + `HAVING`, `ORDER BY`, `LIMIT`, `OFFSET`, `DISTINCT` |
| BE-05 | Support scalar and correlated subqueries |
| BE-06 | Support `TRAIN MODEL` and `PREDICT()` as SQL extensions |
| BE-07 | Parse errors MUST return structured messages with line and column position |

### 2.2 Query Planner & Optimizer

| ID | Requirement |
|----|-------------|
| BE-08 | Transform AST → LogicalPlan → PhysicalPlan |
| BE-09 | Cost-based join ordering using table row-count and column-cardinality statistics |
| BE-10 | Apply predicate pushdown — filter predicates moved as close to scan nodes as possible |
| BE-11 | Select access path per predicate: B+ tree range, hash equality, or full table scan |
| BE-12 | Maintain table statistics; refresh on `INSERT`/`UPDATE`/`DELETE` (approximate) |
| BE-13 | Exact statistics refresh via explicit `ANALYZE <table>` statement |

### 2.3 Execution Engine (Volcano / Iterator Model)

| ID | Requirement |
|----|-------------|
| BE-14 | Implement Volcano iterator model: each operator exposes `open()`, `next() → Option<Row>`, `close()` |
| BE-15 | Physical operators: `SeqScan`, `IndexScan`, `HashJoin`, `MergeJoin`, `NestedLoopJoin` |
| BE-16 | Aggregate operators: `HashAggregate` for `GROUP BY`; functions `SUM`, `COUNT`, `AVG`, `MIN`, `MAX` |
| BE-17 | `Sort` operator with external merge-sort for results exceeding buffer pool |
| BE-18 | `Limit`, `Offset`, `Projection`, `Filter`, `Insert`, `Update`, `Delete` operators |
| BE-19 | `MLPredict` operator integrates model inference into the query plan |

### 2.4 Storage Engine

| ID | Requirement |
|----|-------------|
| BE-20 | Fixed-size 8 KiB pages with slotted-page layout |
| BE-21 | Page header stores: page ID, page type, free-space offset, slot count, LSN |
| BE-22 | Each table stored as a heap file (one `.db` file per table) |
| BE-23 | Record identifiers (RIDs): `(page_id: u32, slot_id: u16)` |
| BE-24 | Deleted records marked with tombstone; physically removed by background vacuum |
| BE-25 | Variable-length columns (`TEXT`, `BLOB`) stored out-of-line when > 512 bytes |

### 2.5 Buffer Pool

| ID | Requirement |
|----|-------------|
| BE-26 | Cache pages in memory; configurable frame count (default 4096 frames = 32 MiB) |
| BE-27 | Eviction policy: LRU-K (K=2) to prevent buffer-pool pollution from large scans |
| BE-28 | Dirty-page table; dirty pages flushed before their WAL records are checkpointed |
| BE-29 | Thread-safe; individual page frames protected by short-duration spinlock latches |

### 2.6 Indexing

| ID | Requirement |
|----|-------------|
| BE-30 | Clustered B+ tree index on primary key for every table |
| BE-31 | Secondary B+ tree indexes on any column or ordered prefix via `CREATE INDEX` |
| BE-32 | B+ tree supports point lookup, range scan, forward/reverse iteration |
| BE-33 | B+ tree nodes fit within a single 8 KiB page; leaf nodes doubly-linked for range scans |
| BE-34 | Hash index for equality predicates (`=`, `IN`); extendible hashing — no full rebuild on grow |
| BE-35 | Hash index buckets are page-aligned (8 KiB) |
| BE-36 | Composite index supports leading-prefix scans |
| BE-37 | Covering index optimization: no heap access when all projected columns are in the index |

### 2.7 Transactions (MVCC)

| ID | Requirement |
|----|-------------|
| BE-38 | MVCC with Snapshot Isolation |
| BE-39 | Monotonically increasing `txn_id: u64` assigned at `BEGIN` |
| BE-40 | Row version header: `xmin: u64`, `xmax: u64`, `cid: u32`, `infomask: u16` |
| BE-41 | Visibility rule: read latest version where `xmin ≤ snapshot` and `xmax > snapshot` |
| BE-42 | Write-write conflicts detected; second writer aborts with `WRITE_CONFLICT` error |
| BE-43 | `COMMIT` fsyncs WAL before returning success |
| BE-44 | `ROLLBACK` undoes writes via WAL undo log |
| BE-45 | Background vacuum reclaims dead row versions no longer visible to any active transaction |

### 2.8 Write-Ahead Log (WAL)

| ID | Requirement |
|----|-------------|
| BE-46 | WAL records written and fsynced before dirty pages are flushed (WAL-before-page invariant) |
| BE-47 | Append-only segment files: `wal-<id>.log`, configurable size (default 64 MiB per segment) |
| BE-48 | WAL record fields: LSN, `txn_id`, record type, before-image, after-image, CRC32 checksum |
| BE-49 | ARIES-style crash recovery on startup: Analysis → Redo → Undo |
| BE-50 | Fuzzy checkpoint every 60 seconds or 1000 dirty pages (whichever first) |
| BE-51 | WAL segments older than the oldest active checkpoint eligible for deletion |

---

## 3. Phase 2 - Networking Layer

### 3.1 HTTP Server

| ID | Requirement |
|----|-------------|
| NET-01 | Single TCP listener; configurable port (default `5433`) |
| NET-02 | Built on `axum` + `tokio`; all I/O non-blocking |
| NET-03 | HTTP/1.1 and HTTP/2 support |
| NET-04 | TLS via `rustls` (no OpenSSL); cert path configurable |
| NET-05 | All request and response bodies in `application/json` |
| NET-06 | Max concurrent connections configurable (default 128) |
| NET-07 | Connection keep-alive; configurable idle timeout (default 60 s) |
| NET-08 | Graceful shutdown: drain in-flight requests; bounded by timeout (default 30 s) |

### 3.2 REST API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/query` | Execute a SQL statement |
| `POST` | `/transaction/begin` | Begin a transaction; returns `txn_id` |
| `POST` | `/transaction/{id}/commit` | Commit a transaction |
| `POST` | `/transaction/{id}/rollback` | Rollback a transaction |
| `GET`  | `/schema/tables` | List all tables |
| `GET`  | `/schema/tables/{table}` | Describe a table's columns and indexes |
| `GET`  | `/ml/models` | List all trained models |
| `DELETE`| `/ml/models/{name}` | Drop a trained model |
| `GET`  | `/health` | Liveness check |
| `GET`  | `/ready` | Readiness check (WAL recovery complete) |
| `GET`  | `/metrics` | Prometheus metrics |

### 3.3 Protocol Details

| ID | Requirement |
|----|-------------|
| NET-09 | Multi-statement transactions: `txn_id` in request body of `/query` |
| NET-10 | Request size limit: 8 MiB (configurable) |
| NET-11 | Response streaming for large result sets (newline-delimited JSON) |
| NET-12 | `X-Request-Id` echoed in every response |
| NET-13 | CORS headers configurable for browser-based frontends |

### 3.4 Error Codes

| Code | HTTP | Meaning |
|------|------|---------|
| `SYNTAX_ERROR` | 400 | SQL parse failure |
| `TYPE_ERROR` | 400 | Type mismatch |
| `TABLE_NOT_FOUND` | 404 | Unknown table |
| `MODEL_NOT_FOUND` | 404 | Unknown ML model |
| `WRITE_CONFLICT` | 409 | MVCC write-write conflict |
| `CONSTRAINT_VIOLATION` | 409 | PK or NOT NULL violated |
| `TXN_NOT_FOUND` | 404 | Unknown transaction ID |
| `INTERNAL_ERROR` | 500 | Engine error |
| `UNAVAILABLE` | 503 | Recovery in progress |

---

## 4. Phase 3 - Security

### 4.1 Authentication

| ID | Requirement |
|----|-------------|
| SEC-01 | API-key auth via `Authorization: Bearer <key>` header |
| SEC-02 | Multiple API keys; each with a role (`admin`, `readwrite`, `readonly`) |
| SEC-03 | Keys stored as Argon2id hashes in `__api_keys` system table |
| SEC-04 | Auth middleware runs before every handler; 401 on failure |
| SEC-05 | Dev mode disables auth and TLS; MUST NOT be used in production |

### 4.2 Authorisation

| ID | Requirement |
|----|-------------|
| SEC-06 | `admin`: full DDL, DML, system tables, ML operations |
| SEC-07 | `readwrite`: DML on user tables, `TRAIN MODEL`, `PREDICT()` |
| SEC-08 | `readonly`: `SELECT` and `PREDICT()` only |
| SEC-09 | Role enforcement at planner level; violations return `PERMISSION_DENIED` |

### 4.3 Transport Security

| ID | Requirement |
|----|-------------|
| SEC-10 | TLS 1.2 minimum; TLS 1.3 preferred; older versions disabled |
| SEC-11 | Only AEAD cipher suites (AES-256-GCM, ChaCha20-Poly1305) |
| SEC-12 | Certificate reload without restart (SIGHUP) |

### 4.4 Data Protection

| ID | Requirement |
|----|-------------|
| SEC-13 | Columns marked `ENCRYPTED` stored as AES-256-GCM ciphertext with per-row IVs |
| SEC-14 | Encryption key via env var `RUSTDB_ENCRYPTION_KEY` (32-byte hex) or KMS reference |
| SEC-15 | WAL records for encrypted columns store ciphertext; keys never written to disk |
| SEC-16 | No index on encrypted columns (v1.0) |

### 4.5 Input Validation

| ID | Requirement |
|----|-------------|
| SEC-17 | All SQL executed through the parser; no raw string interpolation |
| SEC-18 | Parameterised queries supported in `/query` API |
| SEC-19 | Max SQL statement length: 1 MiB |
| SEC-20 | Identifier length capped at 128 bytes |

### 4.6 Audit Logging

| ID | Requirement |
|----|-------------|
| SEC-21 | Every write operation emits a structured audit log entry |
| SEC-22 | Audit entry fields: timestamp, `txn_id`, key fingerprint, statement hash, table |
| SEC-23 | Separate append-only audit log file; not in WAL |
| SEC-24 | Rotation: default 100 MiB per file, retain 30 days |

---

## 5. Phase 4 - Frontend & Client Tools

### 5.1 Web Admin UI

| ID | Requirement |
|----|-------------|
| FE-01 | SPA served from `/ui`; embedded in the RustDB binary via `include_dir!` |
| FE-02 | React + TypeScript + Tailwind CSS; bundled with Vite |
| FE-03 | SQL editor with syntax highlighting (CodeMirror 6) |
| FE-04 | Result grid: sortable columns, pagination, CSV export |
| FE-05 | Schema browser: tables, columns, index metadata, row counts |
| FE-06 | ML model browser: features, target, row count, created date |
| FE-07 | Live metrics dashboard: query rate, error rate, buffer pool hit ratio |
| FE-08 | Dark mode (`prefers-color-scheme`) |
| FE-09 | Auth via login form; API key in `sessionStorage`, cleared on logout |

### 5.2 CLI Tool (`rustdb-cli`)

| ID | Requirement |
|----|-------------|
| FE-10 | Interactive REPL: `rustdb-cli --host --port --key` |
| FE-11 | Multi-line SQL editing with readline history |
| FE-12 | Meta-commands: `\d`, `\d <table>`, `\models`, `\export <file.csv>`, `\timing` |
| FE-13 | Non-interactive mode: `rustdb-cli -c "SELECT 1"` |
| FE-14 | Auto-completion for SQL keywords, table names, column names |

### 5.3 Rust Client SDK (`rustdb-client`)

| ID | Requirement |
|----|-------------|
| FE-15 | Published to crates.io |
| FE-16 | Async API: `Connection::connect(url, key).await` |
| FE-17 | `conn.query(sql).await → ResultSet`; `conn.execute(sql).await → u64` |
| FE-18 | Connection pool: `Pool::new(url, key, max_size)` |
| FE-19 | Transaction builder: `conn.begin().await → Transaction` |
| FE-20 | Row deserialization into user structs via `serde` derive |

### 5.4 Python Client (`rustdb-python`)

| ID | Requirement |
|----|-------------|
| FE-21 | Published to PyPI |
| FE-22 | DB-API 2.0 (PEP 249) compatible |
| FE-23 | `conn.query_df(sql) → pd.DataFrame` |
| FE-24 | Parameterised queries: `cursor.execute("SELECT * FROM t WHERE id = ?", [42])` |

---

## 6. Phase 5 - ML Engine

### 6.1 Supported Algorithms

| ID | Algorithm | Crate | Key Hyperparameters |
|----|-----------|-------|---------------------|
| ML-01 | Linear regression | `linfa-linear` | — |
| ML-02 | Logistic regression | `linfa-logistic` | `max_iter`, `learning_rate` |
| ML-03 | K-means clustering | `linfa-clustering` | `k`, `max_iter` |
| ML-04 | Decision tree | `linfa-trees` | `max_depth`, `min_samples_split` |

### 6.2 Training SQL Syntax

```sql
TRAIN MODEL <name>
  TYPE { linear_regression | logistic_regression | kmeans | decision_tree }
  ON <table> (<feat1>, <feat2>, ...)
  PREDICT <target>
  [ WITH <key> = <value> [, ...] ];
```

| ID | Requirement |
|----|-------------|
| ML-05 | `TRAIN MODEL` runs synchronously within a transaction |
| ML-06 | Training reads features via `SeqScan` into `ndarray::Array2<f64>` |
| ML-07 | Model serialized with `bincode`; stored in `__models` system table |
| ML-08 | Overwrite existing model of same name within the same transaction |
| ML-09 | Non-numeric feature column surfaced as `TYPE_ERROR` at plan time |
| ML-10 | Training progress logged at INFO level |

### 6.3 Inference SQL

```sql
SELECT col1, PREDICT(<model_name>, feat1, feat2, ...) AS pred
FROM <table>;
```

| ID | Requirement |
|----|-------------|
| ML-11 | `PREDICT()` compiles to `MLPredict` physical operator; row-by-row during execution |
| ML-12 | Model deserialized from `__models` and cached (LRU, max 16 models) |
| ML-13 | Feature count and types validated at plan time |
| ML-14 | Output: `FLOAT` for regression, `INT` for classification |

### 6.4 Model Management

| ID | Requirement |
|----|-------------|
| ML-15 | `DROP MODEL <name>` removes from `__models` and inference cache |
| ML-16 | `SHOW MODELS` returns name, algorithm, features, target, row count, created_at |
| ML-17 | Models are transactional; `ROLLBACK` after `TRAIN MODEL` removes the model |

---

## 7. Phase 6 - Optimization & Performance

### 7.1 Query Optimizer

| ID | Requirement |
|----|-------------|
| OPT-01 | Cost model: estimated I/O pages + CPU row cost |
| OPT-02 | Join reordering: dynamic programming ≤ 8 tables; greedy heuristic for > 8 |
| OPT-03 | Predicate pushdown through joins and subqueries |
| OPT-04 | Projection pruning: discard unreferenced columns early |
| OPT-05 | Constant folding at plan time |
| OPT-06 | Index-only scan when all projected columns covered by index |

### 7.2 Storage Optimization

| ID | Requirement |
|----|-------------|
| OPT-07 | Optional per-table LZ4 page compression (default off) |
| OPT-08 | Batch WAL flush for bulk inserts |
| OPT-09 | Read-ahead: prefetch 8 pages ahead for sequential scans |
| OPT-10 | `VACUUM FULL <table>` compacts heap in-place under exclusive lock |

### 7.3 Concurrency Tuning

| ID | Requirement |
|----|-------------|
| OPT-11 | Per-table latch striping: 64 latches per table |
| OPT-12 | Lock-free `txn_id` allocation via `AtomicU64::fetch_add` |
| OPT-13 | MVCC version chain depth bounded (default 10); deep chains trigger background cleanup |

### 7.4 Memory

| ID | Requirement |
|----|-------------|
| OPT-14 | Sort and hash-join spill to disk when working set exceeds configured memory limit |
| OPT-15 | Jemalloc as global allocator for reduced fragmentation |
| OPT-16 | ML feature matrix allocated as single contiguous buffer; released after training |

---

## 8. Phase 7 - Observability & Monitoring

### 8.1 Structured Logging

| ID | Requirement |
|----|-------------|
| OBS-01 | Structured JSON logs via `tracing` + `tracing-subscriber` |
| OBS-02 | Log levels configurable via `RUSTDB_LOG` env var |
| OBS-03 | Per-query log: `txn_id`, `plan_type`, `execution_time_ms`, `rows_returned` |
| OBS-04 | Slow-query log at `WARN` for queries exceeding 500 ms (configurable) |

### 8.2 Prometheus Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `rustdb_queries_total` | Counter | Total queries, labeled by type |
| `rustdb_query_duration_ms` | Histogram | Query latency |
| `rustdb_errors_total` | Counter | Errors by code |
| `rustdb_active_transactions` | Gauge | Open transactions |
| `rustdb_buffer_pool_hit_ratio` | Gauge | Cache hit rate |
| `rustdb_wal_bytes_written_total` | Counter | WAL write throughput |
| `rustdb_ml_predict_latency_ms` | Histogram | Inference latency |

### 8.3 OpenTelemetry Tracing

| ID | Requirement |
|----|-------------|
| OBS-05 | Spans for each query: parse → plan → execute |
| OBS-06 | Span attributes: `db.statement`, `db.rows_affected`, `error` |
| OBS-07 | OTLP gRPC exporter; compatible with Jaeger, Tempo, Honeycomb |
| OBS-08 | Trace sampling rate configurable (default 0.1 in prod) |

---

## 9. Phase 8 - Cloud & Infrastructure

### 9.1 Containerisation

| ID | Requirement |
|----|-------------|
| CLD-01 | Docker image: `ghcr.io/rustdb/rustdb:<version>` |
| CLD-02 | Multi-stage Dockerfile: `rust:1.78-slim` builder + `distroless/cc` runtime |
| CLD-03 | Image size target: < 50 MiB compressed |
| CLD-04 | Non-root user (`rustdb:1000`) inside container |
| CLD-05 | Data at `/data`; WAL at `/data/wal`; all config via env vars |

### 9.2 Kubernetes

| ID | Requirement |
|----|-------------|
| CLD-06 | Helm chart published to Artifact Hub |
| CLD-07 | Chart includes: `Deployment`, `Service`, `PVC`, `ConfigMap`, `Secret`, `ServiceMonitor` |
| CLD-08 | Liveness: `GET /health`; readiness: `GET /ready` |
| CLD-09 | `PodDisruptionBudget`: min 1 available during rolling updates |
| CLD-10 | Default resource requests: 500m CPU / 512Mi RAM; limits: 2 CPU / 4Gi RAM |

### 9.3 Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `RUSTDB_PORT` | `5433` | TCP listen port |
| `RUSTDB_DATA_DIR` | `./data` | Data directory |
| `RUSTDB_WAL_DIR` | `$DATA_DIR/wal` | WAL directory |
| `RUSTDB_BUFFER_POOL_FRAMES` | `4096` | Buffer pool frames |
| `RUSTDB_MAX_CONNECTIONS` | `128` | Max connections |
| `RUSTDB_TLS_CERT` | — | TLS certificate path |
| `RUSTDB_TLS_KEY` | — | TLS key path |
| `RUSTDB_ENCRYPTION_KEY` | — | Column encryption key |
| `RUSTDB_LOG` | `info` | Log filter |
| `RUSTDB_DEV` | `false` | Dev mode |

### 9.4 Backup & Restore

| ID | Requirement |
|----|-------------|
| CLD-11 | `rustdb-backup` CLI: hot backup via WAL shipping to S3-compatible store |
| CLD-12 | Backup = fuzzy checkpoint + WAL segment copy |
| CLD-13 | Point-in-time recovery (PITR): restore snapshot + replay WAL to target LSN |
| CLD-14 | `rustdb-backup verify`: replay into temp instance + run integrity checks |
| CLD-15 | Retention: last 7 daily + 4 weekly (configurable) |

---

## 10. Phase 9 - CI/CD Pipeline

### 10.1 Repository & Branching

| ID | Requirement |
|----|-------------|
| CI-01 | Monorepo: all crates, frontend, CLI, Helm chart, docs |
| CI-02 | Branches: `main` (protected), `dev`, `feature/*`, `hotfix/*` |
| CI-03 | Semantic versioning; `CHANGELOG.md` from conventional commits |
| CI-04 | No direct pushes to `main`; enforced via branch protection |

### 10.2 PR Gate Checks

| ID | Check | Tool |
|----|-------|------|
| CI-05 | Compile (debug + release) | `cargo build` |
| CI-06 | Tests | `cargo test --workspace` |
| CI-07 | Format | `cargo fmt --check` |
| CI-08 | Lint | `cargo clippy -- -D warnings` |
| CI-09 | Security advisories | `cargo audit` |
| CI-10 | Licence compliance | `cargo deny check licenses` |
| CI-11 | Frontend build | `pnpm run build` |
| CI-12 | Frontend type-check | `pnpm run tsc --noEmit` |
| CI-13 | SAST | `semgrep` (Rust + TS rulesets) |
| CI-14 | Container scan | `trivy image` (CRITICAL blocks merge) |

### 10.3 Build Pipeline

| ID | Requirement |
|----|-------------|
| CI-15 | GitHub Actions; self-hosted runners for release builds |
| CI-16 | Release: `cargo build --release` on `x86_64` and `aarch64` |
| CI-17 | Cross-compile targets: `x86_64-linux-musl`, `aarch64-linux-musl`, `x86_64-darwin`, `aarch64-darwin` |
| CI-18 | Build cache: `sccache` on S3, keyed on `Cargo.lock` hash |
| CI-19 | Artifacts: `.tar.gz` binaries + SHA256 checksums on GitHub Releases |
| CI-20 | Docker image pushed to GHCR on merge to `main` and version tags |

### 10.4 Deployment Pipeline

| ID | Requirement |
|----|-------------|
| CI-21 | Environments: `dev` → `staging` → `production` |
| CI-22 | `dev`: auto-deploy on merge to `main` |
| CI-23 | `staging`: auto-deploy on version tag; smoke suite before promotion |
| CI-24 | `production`: manual approval gate after staging passes |
| CI-25 | Helm chart version pinned to app version; rollback via `helm rollback` |
| CI-26 | Migrations applied by init container before server starts |
| CI-27 | Blue-green deployment for zero-downtime upgrades |

---

## 11. Phase 10 - Testing Strategy

### 11.1 Unit Tests

| ID | Requirement |
|----|-------------|
| TST-01 | Every public function in `storage`, `index`, `txn`, `wal` has unit tests |
| TST-02 | B+ tree: insert, lookup, range scan, split, merge, duplicates, concurrent inserts |
| TST-03 | Hash index: insert, lookup, delete, overflow, directory doubling |
| TST-04 | MVCC: visibility rules, write-write conflict, rollback undo, vacuum eligibility |
| TST-05 | WAL: append, flush, recovery replay, checksum validation, segment rotation |
| TST-06 | Buffer pool: fetch, eviction under pressure, dirty-page flush order |
| TST-07 | ML: training convergence, predict shape, serialization round-trip |
| TST-08 | Minimum coverage: **90%** for `storage`, `index`, `txn`, `wal` |

### 11.2 Integration Tests

| ID | Requirement |
|----|-------------|
| TST-09 | End-to-end SQL: SELECT, JOIN, GROUP BY, subqueries vs. expected result sets |
| TST-10 | Transaction lifecycle: BEGIN → DML → COMMIT; durability verified after restart |
| TST-11 | Concurrency: 50 threads × 1000 inserts; no lost updates or dirty reads |
| TST-12 | Crash recovery: SIGKILL mid-transaction; ARIES recovery produces consistent state |
| TST-13 | ML pipeline: TRAIN → PREDICT → DROP; verify output correctness |
| TST-14 | REST API: every endpoint with valid + invalid inputs; auth rejection |

### 11.3 Performance Benchmarks

| ID | Benchmark | Target |
|----|-----------|--------|
| TST-15 | Point lookup — warm cache | < 0.5 ms p99 |
| TST-16 | Point lookup — cold cache | < 2 ms p99 |
| TST-17 | Sequential scan (10M rows) | < 10 s |
| TST-18 | Concurrent inserts (1000 clients) | > 20,000 TPS |
| TST-19 | HashJoin (1M × 1M rows) | < 30 s |
| TST-20 | WAL write throughput | > 100 MB/s |
| TST-21 | PREDICT() inference throughput | > 100k rows/s |
| TST-22 | TRAIN MODEL (100k rows, 5 features) | < 30 s |

Benchmarks via `criterion`; results committed and regressed in CI.

### 11.4 Fuzz Testing

| ID | Requirement |
|----|-------------|
| TST-23 | SQL parser fuzzed with `cargo-fuzz`; no panics on arbitrary input |
| TST-24 | WAL record deserializer fuzzed; no panics on malformed segments |
| TST-25 | Page deserializer fuzzed; no panics on corrupted bytes |
| TST-26 | Fuzz targets run ≥ 24 hours before each minor release |

### 11.5 Security Tests

| ID | Requirement |
|----|-------------|
| TST-27 | SQL injection: verify no injection vector through parameterised query path |
| TST-28 | Auth bypass: every endpoint returns 401 without valid key |
| TST-29 | Role enforcement: `readonly` key + DDL → `PERMISSION_DENIED` |
| TST-30 | Column encryption: ciphertext in WAL, plaintext only via SELECT |
| TST-31 | TLS downgrade: server rejects TLS < 1.2 |

### 11.6 Chaos & Resilience Tests

| ID | Requirement |
|----|-------------|
| TST-32 | Disk-full during WAL write: `STORAGE_FULL` error; no corruption |
| TST-33 | OOM: LRU-K eviction keeps server alive under sustained load |
| TST-34 | Network disconnect mid-transaction: server rolls back after idle timeout |
| TST-35 | SIGKILL under 100 TPS write load; 100% recovery on restart |

### 11.7 Acceptance / Smoke Tests

| ID | Requirement |
|----|-------------|
| TST-36 | Smoke suite < 5 minutes; covers all REST endpoints + basic SQL round-trip |
| TST-37 | Smoke suite auto-runs in `staging` after every deployment |
| TST-38 | Full acceptance suite (integration + benchmarks) runs nightly on `main` |

---

## 12. Non-Functional Requirements

| ID | Category | Requirement |
|----|----------|-------------|
| NFR-01 | Performance | Point lookup p99 < 1 ms (warm cache) |
| NFR-02 | Performance | 10M-row scan < 10 s on NVMe SSD |
| NFR-03 | Durability | Zero committed-transaction loss on hard process crash |
| NFR-04 | Availability | WAL recovery < 30 s for 1 GiB database |
| NFR-05 | Concurrency | Reads never block writes; writes never block reads |
| NFR-06 | Memory | Buffer pool strictly bounded by configured frame count |
| NFR-07 | Safety | All `unsafe` blocks annotated with safety invariant comments |
| NFR-08 | Portability | Linux (x86_64, aarch64) and macOS (Apple Silicon) |
| NFR-09 | Binary size | Release binary < 25 MiB stripped |
| NFR-10 | Startup | Cold start to accepting connections < 3 s |

---

## 13. Data Model Reference

### Supported Types

| SQL Type | Rust | Notes |
|----------|------|-------|
| `INT` | `i32` | 4-byte signed |
| `BIGINT` | `i64` | 8-byte signed |
| `FLOAT` | `f64` | IEEE 754 double |
| `BOOLEAN` | `bool` | 1 byte |
| `VARCHAR(n)` | `String` | UTF-8, max n bytes |
| `TEXT` | `String` | Unbounded; out-of-line if > 512 bytes |
| `TIMESTAMP` | `i64` | µs since Unix epoch |
| `BLOB` | `Vec<u8>` | Out-of-line if > 512 bytes |
| `NULL` | `Option<T>` | Any column nullable unless `NOT NULL` |

### MVCC Row Header

```
[ xmin: u64 ][ xmax: u64 ][ cid: u32 ][ infomask: u16 ]
```

### System Tables

| Table | Purpose |
|-------|---------|
| `__catalog` | Table and column metadata |
| `__indexes` | Index metadata |
| `__models` | Trained ML model blobs |
| `__api_keys` | Hashed API keys and roles |
| `__stats` | Per-table row counts and cardinalities |

---

## 14. Module & Crate Structure

```
rustdb/
├── Cargo.toml                    # workspace root
├── rustdb-server/
│   └── src/
│       ├── main.rs
│       ├── config.rs
│       ├── api/                  # axum HTTP layer
│       │   ├── routes.rs
│       │   ├── handlers.rs
│       │   └── middleware.rs
│       ├── sql/                  # parse → plan → optimize
│       │   ├── parser.rs
│       │   ├── planner.rs
│       │   ├── optimizer.rs
│       │   └── plan.rs
│       ├── executor/             # volcano operators
│       │   ├── seq_scan.rs
│       │   ├── index_scan.rs
│       │   ├── hash_join.rs
│       │   ├── aggregate.rs
│       │   ├── sort.rs
│       │   └── ml_predict.rs
│       ├── storage/
│       │   ├── page.rs
│       │   ├── heap.rs
│       │   ├── buffer_pool.rs
│       │   └── disk_manager.rs
│       ├── index/
│       │   ├── btree/
│       │   ├── hash/
│       │   └── composite.rs
│       ├── txn/
│       │   ├── manager.rs
│       │   ├── mvcc.rs
│       │   └── vacuum.rs
│       ├── wal/
│       │   ├── record.rs
│       │   ├── writer.rs
│       │   ├── reader.rs
│       │   └── checkpoint.rs
│       ├── ml/
│       │   ├── model.rs
│       │   ├── trainer.rs
│       │   ├── inference.rs
│       │   └── algorithms/
│       ├── catalog/
│       ├── security/
│       │   ├── auth.rs
│       │   ├── crypto.rs
│       │   └── audit.rs
│       └── error.rs
├── rustdb-client/                # Rust SDK
├── rustdb-cli/                   # CLI tool
├── rustdb-backup/                # Backup utility
├── frontend/                     # React admin UI
├── migrations/                   # SQL migration scripts
├── helm/                         # Helm chart
├── tests/
│   ├── integration/
│   ├── bench/
│   ├── fuzz/
│   └── smoke/
└── docs/
```

---

## 15. Dependency Manifest

| Crate | Version | Purpose |
|-------|---------|---------|
| `axum` | 0.7 | HTTP server |
| `tokio` | 1 (full) | Async runtime |
| `rustls` | 0.23 | TLS |
| `sqlparser` | 0.43 | SQL parsing |
| `linfa` | 0.7 | ML base |
| `linfa-linear` | 0.7 | Regression |
| `linfa-clustering` | 0.7 | K-means |
| `linfa-trees` | 0.7 | Decision trees |
| `ndarray` | 0.15 | Feature matrices |
| `serde` + `serde_json` | 1 | JSON serialization |
| `bincode` | 2 | Model serialization |
| `thiserror` | 1 | Error types |
| `tracing` + `tracing-subscriber` | 0.1 / 0.3 | Structured logging |
| `opentelemetry-otlp` | 0.15 | Tracing export |
| `prometheus` | 0.13 | Metrics |
| `parking_lot` | 0.12 | RwLock / Mutex |
| `crossbeam` | 0.8 | Lock-free structures |
| `aes-gcm` | 0.10 | Column encryption |
| `argon2` | 0.5 | Key hashing |
| `lz4_flex` | 0.11 | Page compression |
| `lru` | 0.12 | ML model cache |
| `jemallocator` | 0.5 | Allocator |
| `criterion` | 0.5 | Benchmarks |
| `cargo-fuzz` | latest | Fuzz targets |

---

## 16. Glossary

| Term | Definition |
|------|------------|
| **ARIES** | Algorithm for Recovery and Isolation Exploiting Semantics — WAL recovery algorithm |
| **AST** | Abstract Syntax Tree |
| **B+ tree** | Self-balancing index; values in leaf nodes, leaves doubly-linked for range scans |
| **Buffer pool** | In-memory page cache |
| **Covering index** | Index containing all query columns; eliminates heap access |
| **Extendible hashing** | Hash structure that doubles directory without full rebuild |
| **Heap file** | Unordered file of slotted pages storing table rows |
| **LSN** | Log Sequence Number |
| **LRU-K** | Eviction policy tracking K most recent accesses per page |
| **MVCC** | Multi-Version Concurrency Control |
| **PITR** | Point-in-time recovery |
| **RID** | Record Identifier: `(page_id, slot_id)` |
| **Snapshot Isolation** | Reads see consistent snapshot from transaction start |
| **Vacuum** | Background process reclaiming dead row versions |
| **Volcano model** | Pull-based iterator execution model |
| **WAL** | Write-Ahead Log |
| **xmax** | TxnId that deleted a row version (`u64::MAX` if live) |
| **xmin** | TxnId that created a row version |
