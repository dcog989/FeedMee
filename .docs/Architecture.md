# FeedMee – Project Specification

## Overview

**FeedMee** will be a cross-platform RSS/Atom news reader. It will prioritize performance, readability, a clean, responsive and uncluttered UI.

Inspiration: [NetNewsWire](https://netnewswire.com/), [Reeder](https://reederapp.com/), [Feedly](https://feedly.com/news-reader).

---

## Goals

- Deliver a superior reading experience that combats information overload.
- Implement a maintainable, layered architecture using modern coding practices.
- Support flexible article and feed display modes (e.g., cards, magazine, three-pane).
- Ensure scalability for future features (e.g., cloud sync, media support).

---

## Technology Stack

- Vite / Rust / Tauri / Svelte / SQLite

---

## Core Features

### Feed Management

- Add feeds by URL (with auto-discovery from websites) or local import (XML / OPML)
- Organize into nested folders.
- Rename/delete feeds & folders.
- Import/export via OPML.

### Article Handling

- Fetch & parse RSS/Atom feeds.
- Display articles with title, summary, author, image, timestamp.
- Mark as read/unread; archive entries.
- “Inbox” view: unread-only filter.
- tag for reading / ignore

### Reading Experience

- Clean, distraction-free article view.
- Customizable:
  - Font ('Open Sans' default, user configurable)
  - Line spacing (user config)
  - Theme (light/dark/custom themes)

### Settings & UI Preferences

- App theme (light/dark/system)

---

## 7. UI Design Principles

### Layout: Responsive Three-Pane View

The layout adapts fluidly to the window size using modern CSS techniques.

| Window Width   | Behavior                                                                             |
| -------------- | ------------------------------------------------------------------------------------ |
| **>1200px**    | Full three-pane: Navigation → Article List → Reading                                 |
| **768–1200px** | Nav pane collapses to an icon-based rail or hamburger flyout; List + Reading visible |
| **<768px**     | Single-pane navigation: Nav → List → Article (with back button)                      |

#### Pane Details

1. **Navigation Pane**
   - A hierarchical list component displaying folders and feeds.
   - Each item shows a favicon/icon, name, and an unread count indicator.
   - Default width: 280px; user-resizable.

2. **Article List Pane**
   - A virtualized list for smooth scrolling of many articles.
   - Scannable hierarchy for each item:
     - **Primary**: Bold title
     - **Secondary**: Source + author
     - **Tertiary**: Timestamp (right-aligned)
   - Unread indicator: accent-colored dot or bar (left edge).

3. **Reading Pane**
   - A scrollable container with:
     - Max content width: 800px (centered for readability)
     - Generous line height (1.6–1.75)
     - Article title, author, source link, then content
   - Typography: System default UI font for a native feel (user-overridable).

### Modern UI Enhancements

- **Motion**:
  - Subtle fade-in or slide-in animations for list items using Svelte transitions.
  - Smooth, animated transitions between views/pages.
- **Icons**: Vector-based SVG icons (e.g., from a library like Lucide).
- **Type**: Follow a clear typographic scale for visual hierarchy (e.g., Caption, Body, Title).
- **Interaction**:
  - A compact toolbar with icon-only buttons and tooltips on hover.
  - A custom context menu on right-click.
  - Full keyboard support (`Ctrl+R`, `Delete`, arrow nav, etc.).

---

## 8. Technical Implementation

- **Async Operations**: All I/O (network, DB) must be `async`/`await` in Rust and handled via Tauri commands.
- **Error Handling**:
  - Graceful error handling in Rust, propagated to the frontend.
  - User-friendly error messages or toasts in the UI.
- **Migrations**: SQL-based migrations for schema evolution (e.g., using `rusqlite_migration`).
- **Performance**: Rust backend for heavy lifting, virtualized lists in the frontend, and minimal component rendering with Svelte.

---

## 9. Testing Strategy

- **Unit Tests**: Rust services and Svelte components (using Vitest).
- **Integration Tests**: Tauri-specific tests for frontend-backend interaction.
- **UI Tests**: Automated interaction & visual validation (e.g., using Playwright or Cypress).
- **Performance Tests**: Startup time, memory, responsiveness.

---

## 10. Deployment

- TBD (leveraging Tauri's cross-platform bundling capabilities).

---

## 11. Future Work (Out of Scope)

- 2-pane layout option
- Cloud sync (Feedly, Inoreader)
- “Read Later” (Pocket, Instapaper)
- Full-text reader mode
- Podcast/video support
- Priority inbox, duplicate detection
- Keyword/topic following
- Rules, filtering, tagging
