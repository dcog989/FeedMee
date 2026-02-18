# FeedMee Guidelines

FeedMee is a desktop RSS / Atom news feed reader with a clean, minimal style. It aims for fast performance, minimal resource usage.

## Tech Stack

- **Tauri** (v2.9) - Desktop framework wrapping the web frontend
- **Rust** (2024 / v1.93) - Backend logic, Markdown processing, file I/O
- **Svelte** (v5.49) - Frontend framework with Svelte 5 runes (`.svelte.ts` files)
- **TypeScript** (v5.9) - Type-safe frontend code
- **SQLite** (v3.51) - Local database for metadata/bookmarks

## Entry Points

- ?

### Core Components

- ?

### Build Output

- ?

## Key Architecture

### Performance Optimizations

- ?

### Caching Strategy

- ?

### Event Handling

- ?

### Decorator System

- ?

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

- `.assets/`, `.docs/`, `.git/`, `node_modules/`
- `\src-tauri\capabilities`, `\src-tauri\target`, `\src-tauri\gen`, `\src-tauri\Cargo.lock`
- `repomix.config.json`, `.repomixignore`, `bun.lock`
