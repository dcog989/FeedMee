# Agent Directives

## Project Specifics

- Name: FeedMee
- Description: Desktop RSS / Atom news feed reader with a clean, minimal style. Fast performance, minimal resource usage.
- Tech: Tauri (Rust), Svelte 5 (runes), TypeScript, SQLite

### Key Files

- `src-tauri/src/main.rs` — entry point (Rust)
- `src-tauri/src/lib.rs` — command registration (`generate_handler!`)
- `src/routes/+layout.svelte` — entry point (Svelte root layout)
- `src/lib/store.svelte.ts` — state (central, Svelte 5 runes)
- `src/lib/storeTypes.ts` — state interface
- `src/lib/types.ts` — shared data types
- `src/lib/articleActions.svelte.ts` — article list loading/pagination/search
- `src/lib/feedActions.svelte.ts` — feed/folder CRUD
- `src/lib/feedRefresh.svelte.ts` — concurrent feed refresh scheduler
- `src/lib/keyboardNav.svelte.ts` — customizable keyboard shortcuts
- `src/lib/utils/*.test.ts` — tests (vitest, jsdom)
- `src-tauri/src/commands/feeds.rs` — add-feed command (discovery lives in `connectors/`)
- `src-tauri/src/commands/refresh.rs` — single-feed refresh
- `src-tauri/src/commands/crud.rs` — article/folder/feed/tag CRUD, OPML import/export, search
- `src-tauri/src/commands/scraper.rs` — HTML scraping, og:image extraction + backfill
- `src-tauri/src/commands/thumbnails.rs` — thumbnail download/resize/WebP cache
- `src-tauri/src/connectors/bluesky.rs` — Bluesky AT Protocol integration
- `src-tauri/src/connectors/mod.rs` — connector registry
- `src-tauri/src/db.rs` — db module root; schema/migrations in `db/schema.rs`, queries in `db/{articles,feeds,tags}.rs`

### Workflow

- Install: `bun install`
- Dev: `bun run dev`
- Test: `bun run test` (vitest + `cargo test`)
- Lint: `bun run check` (svelte-check, biome, `cargo clippy -D warnings`)
- Format: `bun run format` (biome + `cargo fmt`)
- Build: `bun run build` (`tauri build --no-bundle`)
- Version: `cog bump --auto` (hooks run `bun run check`, `bun run test`, `./scripts/sync_version.sh <version>`, then tag and push)

### Common Patterns

- Add feature: update state in `src/lib/*Actions.svelte.ts`, add UI in `src/lib/components/`, wire it from `src/routes/`.
- Backend call: add the command in `src-tauri/src/commands/`, register it in `src-tauri/src/lib.rs`, then call it with `invoke` from the owning `src/lib/*Actions.svelte.ts` module.
- State access: import the central state from `src/lib/store.svelte.ts`; shared module state, not props drilling.
- These are examples. Follow only if they apply; otherwise state your approach before implementing.

### File System Access

- Allowed: `/home/bubba/Projects/FeedMee/` and all contained directories + files; `/tmp/*`
- Read-Only: `.env*`, `.git/`, `.docs/ToDo.md`
- Disallowed: `node_modules/`, `build/`, `.svelte-kit/`, `src-tauri/target/`, `src-tauri/gen/`, `bun.lock`, `src-tauri/Cargo.lock`
- Require confirmation: adding/removing dependencies, any operation outside project root
- Do not delete files or make destructive changes without permission / confirmation.

### Verification

- Do not run test, lint, format, or type-check commands; the user builds, tests, and lints manually.
- Run them only when the user explicitly asks.

---

## General Guidelines

### Code Changes

- For non-trivial work, propose an approach and confirm before implementing, including architectural changes.
- Keep modifications minimal and scoped; prefer incremental improvements over rewrites.
- Use explicit types and named constants (no magic numbers).
- Return explicit error types; do not suppress exceptions.
- Follow standard repository linting and formatting configs.
- Decompose files over 400 lines if they mix concerns.
- Name things clearly; reserve comments for the non-obvious why (workarounds, edge cases).
- Never run git mutations (commit, push, reset, rebase, amend) unless explicitly instructed.
- Do not create documentation files unless explicitly requested.
- Adhere to <https://develop.kde.org/hig/>.

### Author Environment

- Ryzen 5800X3D, Radeon RX 7900 XT, Display (AW3225QF); CachyOS, KDE, KWin / Wayland, Btrfs.
- fish shell, Ghostty terminal, Fresh TUI editor, yay package manager, bun npm manager, Firefox, Zed code editor.

### Testing

- Add tests only when the logic is genuinely testable and worth guarding. No test files for trivial changes or behavior the environment cannot test reliably (e.g. UI layout/click mapping).

### Definition of Done

- Logic fully implemented.
- Existing docs updated if public interfaces changed.
- Run the corresponding `Workflow` command when `Verification` requires it.

### Commit Messages

- On completion of an update or fix, print a conventional commit message in a fenced code block.
- Add a body only when the "why" isn't obvious.
- Never hard-wrap: one physical line per paragraph, however long.

### Communication Style

- Format: short declarative sentences, one idea per line.
- Instructions: numbered steps, one action each (exact command, path, or code); no prose recap.
- Explanations: essential context only. End question answers with `TL;DR:`, one sentence; omit for status updates and completed work.
- Tone: no preamble, restating, hedging, disclaimers, emotional framing, or closing offers. Keep every fact, command, flag, path, default, caveat.
- Edge Cases: state risks explicitly; ask when requirements are ambiguous.
