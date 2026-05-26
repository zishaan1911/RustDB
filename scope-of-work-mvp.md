## **1\. Problem Statement**

Traditional database architectures separate data storage from machine learning execution. This forces developers to build and maintain brittle, high-latency ETL pipelines that extract data to external Python environments for inference and training. This separation introduces security vulnerabilities, increases the number of moving parts, adding operational complexity, and prevents real-time, data-driven decision-making directly within application queries.

*(For devs, developing this DB enhances our understanding of complex, low-level integration mechanics of storage engines, transaction management, and in-database ML runtimes.)*

## **2\. Minimum Viable Product (MVP) Feature List**

It is suggested that we get the DB work first, which is an MVP, before refining. The following features are proposed:

### **Core Storage & Engine**

* **Slotted Pages & Records:** Fixed 8 KiB disk pages containing variable-length row records, addressed via a simple Record Identifier (page\_id, slot\_id).  
* **Sequential Scan (SeqScan):** A basic Volcano-style iterator that reads rows linearly from a table file (no indexes or joins yet).  
* **Basic SQL Parser:** Minimal execution of CREATE TABLE, INSERT, and SELECT via sqlparser-rs.

### **Transactions & Reliability**

* **Basic MVCC:** Row headers containing tracking fields (xmin, xmax) to support simple Snapshot Isolation visibility checks.  
* **Append-Only WAL:** A lightweight write-ahead log to record transactions sequentially on disk before page flushes, enabling basic crash recovery.

### **Network & Interface**

* **Single Endpoint API:** A lightweight HTTP server (axum) exposing a single POST /query route that accepts raw SQL text and returns responses in JSON format.  
* **Structured Errors:** A unified error handler that intercepts failures and returns clean JSON error objects containing a specific string code (e.g., SYNTAX\_ERROR, WRITE\_CONFLICT, MODEL\_NOT\_FOUND).


*The following features will not be implemented in the MVP*
### **Minimal ML Engine**

* **Sync Training (TRAIN MODEL):** A single, synchronous SQL command that reads a table via SeqScan and trains a linfa Linear Regression model.  
* **In-Query Inference (PREDICT()):** A custom scalar SQL function that executes model inference row-by-row during a SELECT query.  
* **Bincode Serialization:** Save and load trained model weights directly inside a hidden system table (\_\_models).
