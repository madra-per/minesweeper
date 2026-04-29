# Copilot Instructions

## Architecture

This is a Minesweeper game built as a **Tauri v1** desktop app with a **Yew 0.19** (Rust → WebAssembly) frontend.

- **`frontend/`** — The Yew WASM app containing all game logic. Built with [Trunk](https://trunkrs.dev/).
  - `main.rs` — `App` component (Yew `Component` impl), grid management, mine placement, flood-fill reveal (`expand_zero`), and HTML rendering via Yew's `html!` macro.
  - `cell.rs` — `Cellule` struct and `State` enum. Each cell has an `i8` val: `-2` = empty/reset, `-1` = mine, `0..8` = adjacent mine count.
  - `styles.css` — All styling; cell states map to CSS classes (`cellule-hidden`, `cellule-revealed`, `cellule-marked`, `cellule-border`).
- **`src-tauri/`** — Minimal Tauri shell (no custom commands). Only provides the native window wrapper.

The grid uses a 1D `Vec<Cellule>` with a border row/col of `State::Outside` cells to avoid bounds checks on neighbor lookups.

## Build & Run

```sh
# Development (starts Trunk dev server + Tauri window)
cd src-tauri && cargo tauri dev

# Frontend only (WASM served at http://127.0.0.1:8080)
cd frontend && trunk serve

# Production build
cd src-tauri && cargo tauri build
```

**Prerequisites:** Rust toolchain, `wasm32-unknown-unknown` target, Trunk (`cargo install trunk`), Tauri CLI (`cargo install tauri-cli`).

## Conventions

- The codebase uses `Cellule` (not "Cell") to avoid conflicts with `std::cell::Cell`.
- Cell values are encoded as `i8`: `-2` = empty/reset, `-1` = mine, `0..8` = neighbor mine count.
- Mines are generated on first click via `random_mutate`, which guarantees the clicked cell and its neighbors are mine-free.
- Grid coordinates convert between `(row, col)` and a flat index via `row_col_as_idx`; wrapping is used at edges.
