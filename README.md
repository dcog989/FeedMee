# FeedMee

_FeedMee_ is an RSS/Atom news reader built for the desktop.

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

### Prerequisites

1. **Rust:** [Install Rust](https://www.rust-lang.org/tools/install).
2. **Node.js:** [Install Node.js](https://nodejs.org/).
3. **OS Dependencies:** Follow the [Tauri Prerequisites guide](https://v2.tauri.app/start/prerequisites/).

### Installation

```bash
# Install dependencies
bun install
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
