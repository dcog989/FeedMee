# AGENTS.md

FeedMee is a desktop RSS / Atom news feed reader with a clean, minimal style. It aims for fast performance, minimal resource usage.

## Dev Environment

Linux CachyOS, Limine boot loader, KDE Plasma 6, Wayland, Btrfs. Firefox, Kate text editor, Zed code editor, fish shell with Ghostty + Fresh editor. paru and bun package managers. All software is updated as of today.

## Tech Stack

- **Tauri** (v2.11) - Desktop framework wrapping the web frontend
- **Rust** (2024 / v1.95) - Backend logic, web scraping, image processing, file I/O
- **Svelte** (v5.55) - Frontend framework with Svelte 5 runes (`.svelte.ts` files)
- **TypeScript** (v6.0) - Type-safe frontend code
- **SQLite** (v3.53) - Local database for metadata/bookmarks

## Entry Points

- `src-tauri/src/main.rs` - Rust application entry point
- `src/routes/+layout.svelte` - Svelte root layout component

### Core Components

- `src/routes/+page.svelte` - Main UI with 3-pane layout. To be extended in future version.
- `src/lib/components/NavPane.svelte` - Folder/feed navigation
- `src/lib/components/ArticleListPane.svelte` - Article list
- `src/lib/components/ReadingPane.svelte` - Article content reader

- `src/lib/components/TagManager.svelte` - Inline tag add/remove UI
- `src/lib/components/ManageDialog.svelte` - Add feed / OPML / blocked phrases dialog
- `src/lib/components/SettingsModal.svelte` - Settings (refresh, thumbnail, themes, shortcuts)
- `src/lib/components/ReadingPane.svelte` - Article content reader

- `src/lib/store.svelte.ts` - Central state management
- `src/lib/storeTypes.ts` - AppState interface type definitions
- `src/lib/types.ts` - Shared data types (Article, Feed, Folder, Tag, etc.)
- `src/lib/articleActions.svelte.ts` - Article list loading/pagination/search
- `src/lib/feedActions.svelte.ts` - Feed/folder CRUD operations
- `src/lib/feedRefresh.svelte.ts` - Concurrent feed refresh scheduler
- `src/lib/keyboardNav.svelte.ts` - Customizable keyboard shortcuts

### Rust Command Modules

- `src-tauri/src/commands/feeds.rs` - Feed add/discovery (RSS, website, Bluesky)
- `src-tauri/src/commands/refresh.rs` - Feed refresh with og:image backfill
- `src-tauri/src/commands/crud.rs` - Article/folder/tag CRUD
- `src-tauri/src/commands/scraper.rs` - HTML scraping, og:image extraction
- `src-tauri/src/commands/thumbnails.rs` - Thumbnail download, resize, WebP cache
- `src-tauri/src/connectors/bluesky.rs` - Bluesky AT Protocol integration
- `src-tauri/src/connectors/mod.rs` - Connector registry
- `src-tauri/src/db.rs` - SQLite schema, migrations, queries

### Build Output

- `src-tauri/target/` - Rust build artifacts
- `build/` - Svelte compiled frontend (configured in `tauri.conf.json`)

## Key Architecture

### Performance Optimizations

- Pagination (50 articles/page) with infinite scroll
- Concurrent feed refresh (5 workers max)
- Debounced refresh operations (configurable, default 5min/2min)
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

- `/home/bubba/Projects/FeedMee/` unless excluded below.

### Disallowed

- `.assets/`, `.docs/`, `.git/`, `node_modules/`, `.repomix/`
- `/src-tauri/capabilities`, `/src-tauri/target`, `/src-tauri/gen`, `/src-tauri/Cargo.lock`
- `repomix.config.json`, `.repomixignore`, `bun.lock`
