# Distributed Database in Rust

A distributed relational database built from scratch in Rust. Supports SQL queries, crash recovery, indexing, and distributed consensus via the Raft algorithm.

---

## What This Project Does

You can connect to it over TCP and run SQL commands:

```sql
CREATE TABLE users (id INT, name TEXT)
INSERT INTO users VALUES (1, 'Rachit')
SELECT * FROM users WHERE id = 1
UPDATE users SET name = 'Aman' WHERE id = 1
DELETE FROM users WHERE id = 1
CREATE INDEX ON users (id)
```

Data is stored on disk and survives restarts. When running as 3 nodes, all writes are replicated across the cluster using Raft consensus.

---

## Project Structure

```
src/
├── main.rs          ← entry point, starts server
├── type.rs          ← all shared data types (Value, Row, Statement, etc.)
├── parser/mod.rs    ← turns SQL strings into structured data
├── storage/mod.rs   ← reads and writes files to disk
├── executor/mod.rs  ← validates and executes SQL statements
├── wal/mod.rs       ← Write-Ahead Log for crash safety
├── index/mod.rs     ← hash indexes for fast lookups
├── network/mod.rs   ← TCP server, handles client connections
└── raft/
    ├── mod.rs       ← RaftNode, propose, heartbeat, raft listener
    ├── log.rs       ← log entries (term + command)
    ├── rpc.rs       ← message structs (AppendEntries, RequestVote)
    └── election.rs  ← election timer, vote counting
data/
├── users.csv        ← actual row data
├── users.schema     ← column names and types
├── users.wal        ← temporary crash recovery file (deleted after success)
└── users_id.index   ← index file for fast lookups
```

---

## How Data Flows (Single Node)

When you type `INSERT INTO users VALUES (1, 'Rachit')`:

```
Client (nc/telnet)
    │ TCP string
    ▼
network/mod.rs         → reads line from TCP stream
    │
    ▼
parser/mod.rs          → tokenizes and parses the SQL string
    │                    produces: Statement::Insert { table_name, values }
    ▼
executor/mod.rs        → validates types, checks primary key
    │
    ├── wal/mod.rs     → writes "INSERT|1,Rachit" to users.wal  (crash safety)
    ├── storage/mod.rs → appends "1,Rachit" to users.csv        (actual write)
    └── wal/mod.rs     → deletes users.wal                      (cleanup)
    │
    ▼
network/mod.rs         → sends "1 row inserted." back to client
```

---

## Phase by Phase Breakdown

### Phase 1 — Core Engine

**Goal:** Parse and execute basic SQL, store data in files.

**Files:** `type.rs`, `parser/mod.rs`, `storage/mod.rs`, `executor/mod.rs`, `main.rs`

**What was built:**
- `type.rs` defines all shared types: `Value` (Integer/Text/Null), `Row`, `TableSchema`, `Statement`, `QueryResult`
- `parser/mod.rs` tokenizes SQL strings and builds `Statement` structs
- `storage/mod.rs` reads/writes CSV files and schema files to `./data/`
- `executor/mod.rs` validates queries and calls storage
- `main.rs` ran a REPL loop reading from stdin

**File formats:**
```
users.schema:         users.csv:
id:Integer:PK         id,name
name:Text             1,Rachit
                      2,Aman
```

**SQL supported:** `CREATE TABLE`, `INSERT`, `SELECT`, `SELECT ... WHERE`

---

### Phase 2 — Durability & More SQL

**Goal:** Survive crashes, support DELETE/UPDATE, enforce primary keys.

**Files:** `wal/mod.rs`, `executor/mod.rs` (updated)

**Write-Ahead Log (WAL) — crash safety:**

The problem: if your program crashes between writing to disk and finishing, you get corrupt data.

The solution — write in this order:
```
1. Write "INSERT|1,Rachit" to users.wal   ← intention recorded
2. Write row to users.csv                  ← actual write
3. Delete users.wal                        ← success confirmed
```

On startup, `wal::recover()` scans for leftover `.wal` files and replays any incomplete inserts.

**DELETE:**
- Read all rows → filter out matching ones → rewrite entire CSV file

**UPDATE:**
- Read all rows → modify matching ones → rewrite entire CSV file

**Primary Key enforcement:**
- Before inserting, read all rows and check no existing row has the same PK value

---

### Phase 3 — Indexes

**Goal:** Fast lookups without scanning every row.

**Files:** `index/mod.rs`, `executor/mod.rs` (updated)

**The problem:** `SELECT * FROM users WHERE id = 999` scans ALL rows even if there are millions.

**The solution:** A hash map from value → list of row positions:
```
Index for users.id:
  1 → [0]
  2 → [1]
  999 → [5, 8]
```

Now `WHERE id = 999` goes directly to positions 5 and 8 instead of scanning everything.

**Index file format (`users_id.index`):**
```
1:0
2:1
999:5,8
```

**SQL supported:** `CREATE INDEX ON users (id)`, `DROP INDEX ON users (id)`

**How indexes stay up to date:**
- On INSERT → `update_on_insert()` adds new row position to the map
- On DELETE/UPDATE → `rebuild()` rebuilds the entire index from scratch
- On startup → `load()` reads `.index` files back into memory

---

### Phase 4 — TCP Network Server

**Goal:** Accept connections from multiple clients simultaneously.

**Files:** `network/mod.rs`, `main.rs` (updated)

**The problem:** A REPL only works for one person at a time.

**The solution:** TCP server with one thread per client:
```
main() starts TcpListener on 127.0.0.1:7878
    │
    ├── Client 1 connects → spawn thread → handle_client()
    ├── Client 2 connects → spawn thread → handle_client()
    └── Client 3 connects → spawn thread → handle_client()
```

**Thread safety:** All threads share one `Executor` via `Arc<Mutex<Executor>>`:
- `Arc` = shared ownership across threads
- `Mutex` = only one thread can execute a query at a time

**To connect:**
```bash
nc 127.0.0.1 7878
```

---

### Phase 5 — Distributed Raft Consensus

**Goal:** Run on multiple nodes. All nodes stay in sync. If the leader crashes, a new one is elected automatically.

**Files:** `raft/mod.rs`, `raft/log.rs`, `raft/rpc.rs`, `raft/election.rs`, `main.rs` (updated)

**The problem:** With 3 servers, which one handles writes? What if they get different data?

**The solution — Raft algorithm:**
- One node is the **Leader** (handles all writes)
- Other nodes are **Followers** (replicate data from leader)
- If leader crashes → followers **elect a new leader**

**Key concepts:**

**Term:** A logical clock. Every election increments the term. Higher term = more recent authority.

**Log:** Before writing to disk, every command is added to a log. The log is replicated to all nodes first.

**Commit:** A command is only written to storage after a majority of nodes (2 out of 3) confirm they have it in their log.

**How an INSERT works with Raft:**
```
1. Client sends INSERT to Node 1 (Leader)
2. Executor validates the query
3. Raft propose() adds command to leader's log
4. Leader sends "APPEND|1|INSERT|users" to Node 2 and Node 3 over TCP
5. Node 2 and Node 3 append to their logs, reply OK
6. Leader has majority (2/3) → commits → writes to storage
7. Client receives "1 row inserted."
```

**How elections work:**
```
1. Follower timer: no heartbeat in 300ms → start election
2. Increment term, vote for self, become Candidate
3. Send "VOTE|term|id" to all peers over TCP
4. Peers reply "YES" if they haven't voted this term
5. If majority votes YES → become Leader
6. New Leader immediately sends heartbeats every 150ms
```

**Ports:**
```
Node 1: client port 7878, raft port 8878
Node 2: client port 7879, raft port 8879
Node 3: client port 7880, raft port 8880
```

**To run 3 nodes:**
```bash
# Terminal 1
cargo run -- 1 7878 127.0.0.1:8879,127.0.0.1:8880

# Terminal 2
cargo run -- 2 7879 127.0.0.1:8878,127.0.0.1:8880

# Terminal 3
cargo run -- 3 7880 127.0.0.1:8878,127.0.0.1:8879
```

**Connect to the leader (Node 1):**
```bash
nc 127.0.0.1 7878
```

---

## Complete Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│                      CLIENT                              │
└─────────────────────┬───────────────────────────────────┘
                      │ TCP (SQL string)
                      ▼
┌─────────────────────────────────────────────────────────┐
│  network/mod.rs — TCP Server                             │
│  One thread per client, Arc<Mutex<Executor>> shared     │
└─────────────────────┬───────────────────────────────────┘
                      │ Statement enum
                      ▼
┌─────────────────────────────────────────────────────────┐
│  parser/mod.rs — SQL Parser                              │
│  Tokenizer → Parser → Statement                         │
└─────────────────────┬───────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────┐
│  executor/mod.rs — Query Engine                          │
│  Validates types, primary keys, calls storage           │
│                                                         │
│  ┌─────────────┐  ┌──────────┐  ┌──────────────────┐  │
│  │ wal/mod.rs  │  │index/    │  │ raft/mod.rs       │  │
│  │ crash safe  │  │mod.rs    │  │ consensus layer   │  │
│  └─────────────┘  └──────────┘  └──────────────────┘  │
└─────────────────────┬───────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────┐
│  storage/mod.rs — File Storage                           │
│  ./data/users.csv, ./data/users.schema                  │
└─────────────────────────────────────────────────────────┘
```

---

## Running It

**Single node:**
```bash
cargo run -- 1 7878
nc 127.0.0.1 7878
```

**Three nodes:**
```bash
cargo run -- 1 7878 127.0.0.1:8879,127.0.0.1:8880
cargo run -- 2 7879 127.0.0.1:8878,127.0.0.1:8880
cargo run -- 3 7880 127.0.0.1:8878,127.0.0.1:8879
nc 127.0.0.1 7878
```

**Example session:**
```
CREATE TABLE users (id INT PRIMARY KEY, name TEXT)
Table created.

INSERT INTO users VALUES (1, 'Rachit')
1 row inserted.

INSERT INTO users VALUES (2, 'Aman')
1 row inserted.

SELECT * FROM users
id | name
---------
1  | Rachit
2  | Aman
(2 rows)

CREATE INDEX ON users (id)
Index created.

SELECT * FROM users WHERE id = 1
id | name
---------
1  | Rachit
(1 row)

UPDATE users SET name = 'Bob' WHERE id = 2
1 row(s) updated.

DELETE FROM users WHERE id = 1
1 row(s) deleted.
```
