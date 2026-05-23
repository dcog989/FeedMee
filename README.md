# FeedMee

_FeedMee_ is an RSS/Atom news reader built for the desktop.

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
- **Smart Views:** "Latest" (24h) and "Read Later" (Saved) aggregation.
- **Local Privacy:** All data is stored locally in SQLite. No tracking, no accounts.
- **Customizable:** Dark/Light themes, configurable refresh intervals, and log rotation.

## Tech Stack

- **Frontend:** Svelte 5 / Runes, TypeScript, Vite
- **Backend:** Rust, Rusqlite (SQLite), Reqwest, Feed-RS
- **Build System:** Tauri v2

## Getting Started

### Install on Arch Linux

```bash
bun run package:arch
```

This runs `makepkg -si` from `.pkg/`, which compiles and installs FeedMee to `/usr/bin/feedmee`.

**Build dependencies:** `rust`, `bun`, `npm`, `sqlite`, `cmake`, `nasm`

**Runtime dependencies:** `webkit2gtk-4.1`, `gtk3`, `libayatana-appindicator`, `sqlite`

### Build from Source

**Prerequisites:**

1. **Rust:** [Install Rust](https://www.rust-lang.org/tools/install)
2. **Bun:** [Install Bun](https://bun.sh)
3. **OS Dependencies:** Follow the [Tauri Prerequisites guide](https://v2.tauri.app/start/prerequisites/)

```bash
# Install dependencies
bun install

# Run in development
bun run dev

# Build release binary
bun run tauri build
```

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
