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
- `src-tauri/src/commands/feeds.rs` — feed discovery/add
- `src-tauri/src/commands/refresh.rs` — feed refresh + og:image backfill
- `src-tauri/src/commands/crud.rs` — article/folder/tag CRUD
- `src-tauri/src/commands/scraper.rs` — HTML scraping, og:image extraction
- `src-tauri/src/commands/thumbnails.rs` — thumbnail download/resize/WebP cache
- `src-tauri/src/connectors/bluesky.rs` — Bluesky AT Protocol integration
- `src-tauri/src/connectors/mod.rs` — connector registry
- `src-tauri/src/db.rs` — SQLite schema, migrations, queries

### Workflow

- Install: `bun install`
- Dev: `bun run dev`
- Test: `bun run test` (vitest + `cargo test`)
- Lint: `bun run check` (svelte-check, biome, `cargo clippy -D warnings`)
- Format: `bun run format` (biome + `cargo fmt`)
- Build: `bun run build` (`tauri build --no-bundle`)
- Version: `bun run release` (auto) or `bun run version <x.y.z>`; Cocogitto syncs manifests and tags

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

- For non-trivial work, propose an approach and confirm before implementing.
- Keep modifications minimal and scoped; prefer incremental improvements over rewrites. Ask before architectural changes.
- Use explicit types and named constants (no magic numbers).
- Return explicit error types; do not suppress exceptions.
- Follow standard repository linting and formatting configs.
- Decompose files over 400 lines if they mix concerns.
- Use clear naming over comments; reserve comments for complex workarounds or non-obvious issues — why, not what.
- Never run git mutations (commit, push, reset, rebase, amend) unless explicitly instructed.
- Do not create documentation files unless explicitly requested.

### Author Environment

- CachyOS, KDE Plasma 6, Wayland, Btrfs.
- fish shell, Ghostty terminal, Fresh TUI editor, yay package manager, bun npm manager, Firefox, and Zed code editor.

### Testing

- Do not create test files for trivial changes, or for behavior that is not reliably unit-testable in the test environment (e.g. UI layout/click mapping). Prefer no new files; only add a test when the logic is genuinely testable and worth guarding.

### Definition of Done

- Logic fully implemented.
- Existing docs updated if public interfaces changed.
- When required by the `Verification` rules, run the corresponding `Workflow` command.

### Commit Messages

- On completion of an update or fix, print a conventional commit message in a fenced code block.
- Add a body only when the "why" isn't obvious.
- Do not wrap lines. One physical line per paragraph however long it gets. Never hard-wrap the subject or the body.

### Communication Style

- Terse: short declarative sentences, one idea per line.
- Fixes and instructions: numbered steps, one action each, exact command/path/code. No prose paragraphs or recap.
- Questions and explanations: direct prose, only the reasoning needed.
- Cut words, never facts — keep every command, flag, path, default and caveat.
- No preamble, restating, hedging, disclaimers, meta-commentary, emotional framing, or closing offers; end when done unless a real decision is needed.
- Ask when requirements are ambiguous; state risks and edge cases.
- End answers to questions with a `TL;DR:` line: one sentence, the whole point. Skip it for acknowledgements, status updates, and completed work.
