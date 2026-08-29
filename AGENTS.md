# Agent Directives

## Project Context

- Name: FeedMee
- Description: Desktop RSS / Atom news feed reader with a clean, minimal style. Fast performance, minimal resource usage.
- Tech: Tauri (v2.11), Rust (2024 / v1.95), Svelte (v5.56), TypeScript (v6.0), SQLite (v3.53)

## Key Files

- `src-tauri/src/main.rs` — Rust entry point
- `src/routes/+layout.svelte` — Svelte root layout
- `src/lib/store.svelte.ts` — Central state (Svelte 5 runes)
- `src/lib/storeTypes.ts` — AppState interface
- `src/lib/types.ts` — Shared data types
- `src/lib/articleActions.svelte.ts` — Article list loading/pagination/search
- `src/lib/feedActions.svelte.ts` — Feed/folder CRUD
- `src/lib/feedRefresh.svelte.ts` — Concurrent feed refresh scheduler
- `src/lib/keyboardNav.svelte.ts` — Customizable keyboard shortcuts
- `src-tauri/src/commands/feeds.rs` — Feed discovery/add
- `src-tauri/src/commands/refresh.rs` — Feed refresh + og:image backfill
- `src-tauri/src/commands/crud.rs` — Article/folder/tag CRUD
- `src-tauri/src/commands/scraper.rs` — HTML scraping, og:image extraction
- `src-tauri/src/commands/thumbnails.rs` — Thumbnail download/resize/WebP cache
- `src-tauri/src/connectors/bluesky.rs` — Bluesky AT Protocol integration
- `src-tauri/src/connectors/mod.rs` — Connector registry
- `src-tauri/src/db.rs` — SQLite schema, migrations, queries

---

## Development Workflow

- **install**: `bun install`
- **dev**: `bun run dev`
- **test**: (none yet)
- **lint**: `bun run check` (types + frontend + backend) or individually: `bun run lint:types`, `bun run lint:frontend`, `bun run lint:backend`
- **format**: `bun run format`
- **build**: `bun run build`
- **version**: `bun run release` (auto), `bun run version 1.2.3` (explicit); cocogitto syncs manifests, changelog, and tags `v*` then pushes (triggers GitHub release)
- **commits**: Conventional Commits, enforced by cocogitto via lefthook (`commit-msg` hook) and CI. Types: `feat|fix|refactor|chore|docs|style|perf|test|build|ci|revert`.

## File System Access

- Root: `/home/bubba/Projects/FeedMee/`
- Allowed: All subdirectories, `/tmp/*`
- Read-Only: `.env*`, `.git/`
- Disallowed: `.assets/`, `.docs/`, `.git/`, `node_modules/`, `.repomix/`, `src-tauri/capabilities`, `src-tauri/target`, `src-tauri/gen`, `src-tauri/Cargo.lock`, `repomix.config.json`, `.repomixignore`, `bun.lock`
- Require confirmation: adding/removing dependencies, changes outside `src/`, any operation outside project root

## Rules

- Keep modifications minimal and scoped. Ask before architectural changes.
- Do not delete files or make destructive changes without confirmation.
- Do not create documentation files unless explicitly requested.
- Prefer incremental improvements over rewrites.
- KISS, DRY, YAGNI, SoC, SOLID, Composition Over Inheritance, Rule of Three, POLA, Fail Fast.
- Optimize for actual and perceived performance.
- Use explicit types and named constants (no magic numbers).
- Return explicit error types; do not suppress exceptions.
- Follow standard repository linting and formatting configs (Biome, rustfmt, .editorconfig).
- Decompose files over 400 lines if they mix concerns.
- Self-documenting code via clear naming. Use comments only for complex workarounds or issues that need noting.
- Never run git mutations (commit, push, reset, rebase, amend) unless explicitly asked.
- Do not run full `bun run check`/`bun run test` on trivial changes (constant tweaks, one-line edits, CSS value changes). Run `bunx biome check --write <file>` on the touched file, or nothing if the change is a simple value edit. Only run the full suite on real logic changes.
- On completion of an update or fix, print a concise conventional commit message in a fenced code block.

## Communication Style

- Provide concise, actionable responses. No analogies.
- Ask clarifying questions when requirements are ambiguous.
- Flag potential risks or edge cases proactively.
- Do not pretend to understand how the user feels. Do not pretend to be human.

## Definition of Done

- Logic fully implemented.
- `bun run check` passes with zero errors.
- New/modified features have tests (when test framework is added).
- Existing docs updated if public interfaces changed.
