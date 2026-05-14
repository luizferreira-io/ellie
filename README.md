# Ellie

A terminal-based PostgreSQL performance monitoring and tuning tool.

Ellie connects to a PostgreSQL server and provides a real-time interactive dashboard for visualizing server health, active sessions, configuration settings, and detailed performance analysis — all from your terminal.

The name "Ellie" comes from the female mammoth character in the Ice Age movies.

---

## Screenshots

![Screenshot 01](doc/screenshots/screenshot-01.png)


![Screenshot 02](doc/screenshots/screenshot-02.png)


![Screenshot 05](doc/screenshots/screenshot-05.png)


![Screenshot 09](doc/screenshots/screenshot-09.png)


![Screenshot 11](doc/screenshots/screenshot-11.png)

More screenshots [here](https://github.com/luizferreira-io/ellie/tree/main/doc/screenshots).

---

## Features

- **Dashboard** — Real-time server metrics: cache hit ratio, transactions, rollbacks, locks, conflicts, active sessions, and shared buffers usage.
- **Activity** — Live view of all active connections, running queries, wait events, and client details.
- **Settings** — Browse all PostgreSQL configuration parameters with their current values, units, and descriptions.
- **File Settings** — View settings as loaded from the server's configuration files.
- **Tuning** — 18 structured performance analyses across five categories: disk allocation, table fragmentation, indexing, shared buffers, and query performance.
- **About** — Application information and credits.
- **UI** — Text user interface (TUI). It runs directly in the terminal. Browse tables, adjust column widths, reposition columns.

---

## Requirements

- PostgreSQL 10 or later.
- A database user with read access to system catalog views (`pg_stat_*`, `pg_class`, etc.)
- For full tuning functionality, the following PostgreSQL extensions must be enabled:
  - [`pg_buffercache`](https://www.postgresql.org/docs/current/pgbuffercache.html) — required for Shared Buffers analyses
  - [`pg_stat_statements`](https://www.postgresql.org/docs/current/pgstatstatements.html) — required for Time-Consuming Queries analyses

To activate these extensions, follow the steps below:

1. Run in the SQL console:
```SQL
ALTER SYSTEM SET shared_preload_libraries = 'pg_stat_statements';
```
2. Restart the server instance.
3. Run in the SQL console:
```SQL
CREATE EXTENSION IF NOT EXISTS pg_stat_statements;
CREATE EXTENSION IF NOT EXISTS pg_buffercache;
```
4. Tests whether the views exists:
```SQL
SELECT * FROM pg_stat_statements;
SELECT * FROM pg_buffercache;
```
---


## Usage

```
ellie [OPTIONS]
```

### Options

| Option | Default | Description |
|---|---|---|
| `--url <url>` | — | Full PostgreSQL connection URL. Overrides all other parameters. |
| `--host <host>` | `localhost` | Server hostname or IP address. |
| `--port <port>` | `5432` | Server port. |
| `--user <user>` | `postgres` | Database user. |
| `--password <password>` | `postgres` | User password. |
| `--database <database>` | `postgres` | Database name. |
| `--help` | — | Show help. |

### Examples

```bash
# Connect using individual parameters
ellie --host db.example.com --port 5432 --user admin --password secret --database mydb

# Connect using a URL
ellie --url postgresql://admin:secret@db.example.com:5432/mydb
```

---

## Installing

Just unzip it and run it in the terminal.

Pre-compiled binaries for 64-bit Linux and Windows are available in [releases](https://github.com/luizferreira-io/ellie/releases).

---

## Building on Linux

You will need the Rust compiler. If you don't have it, follow these instructions: https://rust-lang.org/tools/install/

Install dev dependencies:

```bash
sudo apt install build-essential
```

Clone the source code:

```bash
git clone https://github.com/luizferreira-io/ellie
cd ellie
```

Build it:

```bash
cargo build --release
```

The binary will be located at `target/release/ellie`.

---

If you want to build with muls ABI (for maximum compatibility and portability), install musl dev dependencies:

```bash
sudo apt-get install musl-tools musl-dev
```

Add the target:
```bash
rustup target add x86_64-unknown-linux-musl
```

And build it:
```bash
cargo build --release --target x86_64-unknown-linux-musl
```

The binary will be located at `target/x86_64-unknown-linux-musl/release/ellie`.

---

## Building on Windows

You will need the Rust compiler. If you don't have it, follow these instructions: https://rust-lang.org/tools/install/

Clone the source code:

```bash
git clone https://github.com/luizferreira-io/ellie
cd ellie
```

Build it:

```bash
cargo build --release
```

The binary will be located at `target\release\ellie.exe`.

This is a **terminal application**. Even on Windows, open terminal first (PowerShell, etc.), and only then run it.

---

## Testing

With Rust installed, just run:
```bash
cargo test
```

---

## Backlog

### Features

- **Connection profiles** — Save and switch between multiple named server connections without re-entering credentials each time.
- **Configuration file** — Support a `~/.config/ellie/config.toml` for storing default connection parameters.
- **Search and filter** — Add inline search (`/`) to filter rows in the Settings, File Settings, and Activity tabs.
- **Copy to clipboard** — Allow copying the selected row or cell value from any table.
- **Export** — Export tuning analysis results to CSV or plain text.
- **Threshold alerts** — Highlight metrics in the Dashboard when they exceed user-defined thresholds (e.g., cache hit ratio below 90%).
- **Additional tuning analyses** — Expand the Tuning tab with analyses for replication lag, autovacuum status, and connection saturation.
- **Activity** — Kill active session/query.

### Improvements

- **Reconnect on disconnect** — Detect lost connections and attempt automatic reconnection instead of crashing.
- **Configurable refresh interval** — Let the user adjust the data refresh rate from the Settings tab or via a CLI flag.
- **Help overlay** — Add an in-app `?` keybinding that shows a reference of all shortcuts for the current tab.
- **New tabs** - Database explorer, query executor/explainer/analyser, and troubleshooting.

---

## Sample Database

The database used for the screenshots and tests is **dvdrental**, available at:

https://neon.com/postgresql/getting-started/sample-database
