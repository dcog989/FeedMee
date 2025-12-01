# FeedMee

*FeedMee* is an RSS/Atom news reader built for the desktop. This is the initial, beta release. More to come.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Tauri](https://img.shields.io/badge/built%20with-Tauri-24C8DB.svg)
![Svelte](https://img.shields.io/badge/frontend-Svelte%205-FF3E00.svg)

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

- **Frontend:** Svelte 5, TypeScript, Vite
- **Backend:** Rust, Rusqlite (SQLite), Reqwest, Feed-RS
- **Build System:** Tauri v2

## Getting Started

### Prerequisites

1.  **Rust:** [Install Rust](https://www.rust-lang.org/tools/install).
2.  **Node.js:** [Install Node.js](https://nodejs.org/).
3.  **OS Dependencies:** Follow the [Tauri Prerequisites guide](https://v2.tauri.app/start/prerequisites/).

### Installation

```bash
# Install dependencies
npm install
