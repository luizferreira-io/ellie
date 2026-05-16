# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Ellie is a terminal-based PostgreSQL performance monitoring and tuning tool built in Rust. It connects to a PostgreSQL server and renders a real-time TUI dashboard using [ratatui](https://github.com/ratatui-org/ratatui).

## Commands

```bash
# Build (debug)
cargo build

# Build (release, optimized)
cargo build --release

# Build for musl (portable Linux binary)
cargo build --release --target x86_64-unknown-linux-musl

# Run tests
cargo test

# Run a single test by name
cargo test <test_name>

# Lint
cargo clippy

# Run (requires a running PostgreSQL server)
cargo run -- --host localhost --port 5432 --user postgres --password postgres --database postgres
cargo run -- --url postgresql://user:pass@host:5432/dbname
```

MSRV: **1.85** (Rust 2024 edition).

## Architecture

### Entry point and main loop

`main.rs` installs a panic hook that restores the terminal before printing, then delegates to `App::new().init().run()`.

`app.rs` — `App` owns:
- A `Vec<Box<dyn Tab>>` with all six tabs (Dashboard, Activity, Settings, File Settings, Tuning, About)
- The active `postgres::Client`
- Current tab index and last error string

The event loop polls for input every 250 ms, dispatches key events to the active tab, and calls `needs_refresh()` on that tab to decide whether to re-query PostgreSQL.

### Tab trait

`src/tabs/tab.rs` defines the `Tab` trait:
- `get_name()` / `get_footer()` — header/footer strings
- `render(frame, area)` — draw the tab
- `handle_event(keycode)` — handle keyboard input
- `update_data(client)` — run SQL queries and update internal state
- `needs_refresh()` — return true when a refresh timer has elapsed

Each tab struct in `src/tabs/tab_*.rs` implements this trait and owns its data state and widget instances.

### Database layer

`src/database/mod.rs` provides:
- `db_connect(args)` — tries TLS first (with `AcceptAllVerifier` — accepts all certs), falls back to `NoTls`
- `db_query(client, query)` — thin wrapper around `client.query(query, &[])`
- Static query maps (`QUERIES_DASHBOARD`, `QUERIES_ACTIVITY`, etc.) keyed by tab-specific enums (`DashboardKey`, `ActivityKey`, etc.)

Each `queries_*.rs` file defines a map keyed by an enum, where each entry carries a `.query` string.

### Widgets

`src/widgets/` has two layers:
- **Primitive widgets** (`widget_chart`, `widget_table`, `widget_menu`, `widget_summary`, `widget_text_box`, `widget_section_title`, `widget_simple_table`) — reusable ratatui widget wrappers
- **Sections** (`section_activity`, `section_instance`, `section_sessions`, `section_shared_buffers`) — composite widgets used inside `TabDashboard`, each holding multiple primitives and rendering a logical dashboard panel

`WidgetTable` (`widget_table.rs`) is the most complex widget: it supports column width adjustment, column reordering, horizontal scrolling, and row selection, all managed through an internal `CompleteScrollState`.

### MetricHistory

`src/tabs/metric_history.rs` — a fixed-capacity ring buffer (`VecDeque<f64>`) for time-series chart data. Three push modes:
- `push_delta(raw)` — stores per-second rate (delta / interval); first call is consumed silently
- `push_absolute(raw)` — stores raw value as-is
- `push_value(f64)` — stores a pre-computed float

`chart_data()` returns `Vec<(f64, f64)>` with x values offset so the right edge is always at `capacity - 1`, keeping charts visually stable as data arrives.

### Color palette

All colors are constants in `src/palette.rs`, imported as `use crate::palette as P`. Never hardcode colors inline.

### Global key bindings

| Key | Action |
|---|---|
| `[` / `]` | Previous / next tab |
| `1`–`9` | Jump to tab by number |
| `q`, `Q`, `Esc` | Quit |

Per-tab bindings are handled in each tab's `handle_event`.
