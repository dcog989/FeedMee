# FeedMee Guidelines

FeedMee is a desktop RSS / Atom news feed reader with a clean, minimal style. It aims for fast performance, minimal resource usage.

## Tech Stack

- **Tauri** (v2.9) - Desktop framework wrapping the web frontend
- **Rust** (2024 / v1.93) - Backend logic, Markdown processing, file I/O
- **Svelte** (v5.49) - Frontend framework with Svelte 5 runes (`.svelte.ts` files)
- **TypeScript** (v5.9) - Type-safe frontend code
- **SQLite** (v3.51) - Local database for metadata/bookmarks

## Entry Points

- `src-tauri/src/main.rs` - Rust application entry point
- `src/routes/+layout.svelte` - Svelte root layout component

### Core Components

- `src/routes/+page.svelte` - Main UI with 3-pane layout
- `src/lib/components/NavPane.svelte` - Folder/feed navigation
- `src/lib/components/ArticleListPane.svelte` - Article list
- `src/lib/components/ReadingPane.svelte` - Article content reader
- `src/lib/components/TitleBar.svelte` - Custom window title bar
- `src/lib/store.svelte.ts` - Central state management

### Build Output

- `src-tauri/target/` - Rust build artifacts
- `build/` - Svelte compiled frontend (configured in `tauri.conf.json`)

## Key Architecture

### Performance Optimizations

- Pagination (50 articles/page) with infinite scroll
- Concurrent feed refresh (2 workers max)
- Debounced refresh operations (configurable, default 5min/2min)
- Virtualized rendering for large lists
- Auto-vacuum every 24 hours

### Caching Strategy

- In-memory state via Svelte 5 runes in `store.svelte.ts`
- LocalStorage for UI preferences (nav width, sort order)
- SQLite for persistent data (feeds, articles, read/saved status)
- Log rotation (5 files max) in app data directory

### Event Handling

- Tauri invoke commands for all backend operations
- UI updates via reactive `$state` and `$effect`
- Modal system for confirmations/alerts
- Tooltip system via Svelte actions (`tooltip.svelte.ts`)

### Decorator System

- None - uses plain Svelte 5 components with TypeScript

## Coding Principles

- Use current coding standards and patterns (Svelte 5 runes, modern TS/Rust)
- KISS, Occam's razor, DRY, YAGNI
- Optimize for actual and perceived performance
- Self-documenting code via clear naming
- Comments only for workarounds/complex logic - do NOT add comments as running dev commentary.
- No magic numbers
- Split files of 400+ lines in to separate distinct functions
- **Do NOT create docs files** (summary, reference, testing, etc.) unless explicitly requested

## File System Access

### Allowed

- all root folders and files unless excluded below.

### Disallowed

- `.assets/`, `.docs/`, `.git/`, `node_modules/`, `.repomix/`
- `/src-tauri/capabilities`, `/src-tauri/target`, `/src-tauri/gen`, `/src-tauri/Cargo.lock`
- `repomix.config.json`, `.repomixignore`, `bun.lock`
