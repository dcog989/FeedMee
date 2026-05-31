# README: FeedMee

_FeedMee_ is an RSS/Atom news reader built for the desktop, cross-platform and Linux-first.

[Download from Latest Releases](https://github.com/dcog989/FeedMee/releases).

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Tauri](https://img.shields.io/badge/built%20with-Tauri-24C8DB.svg)
![Svelte](https://img.shields.io/badge/frontend-Svelte%205-FF3E00.svg)

![screen 1](/assets/screen1.webp)

## Features

- **Cross-Platform:** Native performance on Windows, macOS, and Linux (via Tauri v2).
- **Three-Pane Layout:** Classic, responsive interface (Navigation, Article List, Reading Pane).
- **Reader Mode:** Extracts full article content using `Readability`, stripping clutter.
- **Feed Management:**
  - Auto-discovery of RSS/Atom links from URLs.
  - Drag-and-drop folder organization.
  - OPML Import/Export.
  - Bluesky profile support.
- **Smart Views:** "Latest" (24h) and "Read Later" (Saved) aggregation.
- **Article Tagging:** Color-coded tags for categorization and filtering.
- **Thumbnails:** Automatic og:image extraction with resizing, WebP caching, and configurable size.
- **Local Privacy:** All data is stored locally in SQLite. No tracking, no accounts.
- **Customizable:** Dark/Light themes, configurable refresh intervals, thumbnail size, and log rotation.

## Tech Stack

- **Frontend:** Svelte 5 / Runes, TypeScript, Vite
- **Backend:** Rust, Rusqlite (SQLite), Reqwest, Feed-RS
- **Build System:** Tauri v2

## Getting Started

### Prerequisites

1. **Rust:** [Install Rust](https://www.rust-lang.org/tools/install)
2. **Bun:** [Install Bun](https://bun.sh)
3. **OS Dependencies:** Follow the [Tauri Prerequisites guide](https://v2.tauri.app/start/prerequisites/)

```bash
bun install
```

### Development

```bash
bun run dev
```

Starts the Vite dev server and Tauri with HMR.

### Install on Arch / CachyOS

```bash
bun run package
```

Runs `makepkg -si` from `.pkg/`, compiling from source and installing FeedMee to `/usr/bin/feedmee`.

**Build dependencies:** `rust`, `bun`, `npm`, `sqlite`, `cmake`, `nasm`

**Runtime dependencies:** `webkit2gtk-4.1`, `gtk3`, `libayatana-appindicator`, `sqlite`

### Build Release Binary

```bash
bun run build
```

Compiles the Rust backend and Svelte frontend. Output binary at `src-tauri/target/release/FeedMee`. No installer is produced locally — AppImage and Windows installer are built via GitHub Actions on tag push.

### Validate

```bash
bun run check
```

Runs TypeScript, Svelte, and Rust lint checks. Use `bun run check:watch` to keep running in a separate terminal during development.

## Keyboard Shortcuts

| Key     | Action                     |
| ------- | -------------------------- |
| `/`     | Focus search               |
| `r`     | Refresh all feeds          |
| `n`     | Add new feed               |
| `,`     | Open settings              |
| `s`     | Save/Read later            |
| `m`     | Mark as read/unread        |
| `Enter` | Open article in browser    |
| `x`     | Expand all folders         |
| `c`     | Collapse all folders       |
| `Esc`   | Close modal / Clear search |

> Tip: Click the keyboard icon in Settings to customize shortcuts.
