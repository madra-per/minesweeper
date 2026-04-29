# Minesweeper

A classic Minesweeper game built as a desktop app with [Tauri](https://tauri.app/) and [Yew](https://yew.rs/) (Rust → WebAssembly).

Adapted from the [Yew Game of Life example](https://github.com/yewstack/yew/tree/master/examples/game_of_life).

## Prerequisites

- [Rust](https://rustup.rs/) with the `wasm32-unknown-unknown` target
- [Trunk](https://trunkrs.dev/) — `cargo install trunk`
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites) — `cargo install tauri-cli`

## Getting Started

```sh
# Run the full desktop app (Trunk dev server + Tauri window)
cd src-tauri && cargo tauri dev

# Run the frontend only in a browser (http://127.0.0.1:8080)
cd frontend && trunk serve
```

## Building for Production

```sh
cd src-tauri && cargo tauri build
```

## How to Play

- **Left-click** a cell to reveal it. The first click generates the minefield (the clicked cell and its neighbors are always safe).
- **Right-click** a cell to mark/unmark it as a suspected mine.
- Reveal all non-mine cells to win. Hit a mine and it's game over.
- Click **Start** or **Reset** to begin a new game.