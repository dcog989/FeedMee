# Review FeedMee

THe app is split in to the following chunks to allow optimal review:

## Repo / config / docs (4)

| # | Chunk | Files | LOC | Focus |
| --- | ------- | ------- | ----- | ------- |
| N1 | Build and dependency config | `package.json`, `src-tauri/Cargo.toml`, `tauri.conf.json`, `capabilities/default.json`, `build.rs`, `rust-toolchain.toml`, `.cargo/config.toml`, `deny.toml`, `tsconfig.json`, `vite.config.js`, `vitest.config.ts`, `biome.json`, `.editorconfig`, `.rustfmt.toml` | 417 | Feature flags vs code use, CSP null, bundle targets, permissions scope, dependency freshness, dev/build URL consistency |
| N2 | Release and version automation | `cog.toml`, `changelog.tpl`, `scripts/sync_version.sh`, `PKGBUILD`, `feedmee.desktop`, `.github/workflows/ci.yml`, `.github/workflows/release.yml`, `dependabot.yml`, `lefthook.yml` | 377 | Version sync targets, hook ordering, signing/pubkey mismatch, artifact names, CI coverage gaps |
| N3 | Shell styling and assets | `src/app.css`, `src/app.html`, `src-tauri/icons/icon.png`, `static/feedmee.png`, `assets/screen-1.webp` | 184 + images | Theme variables, layout resets, icon sizes/format, image weight, screenshot accuracy |
| N4 | Docs and repo metadata | `README.md`, `AGENTS.md`, `LICENSE`, `.gitignore`, `.gitattributes` | 930 | Claims vs code, ignored paths, line-ending rules |

## Backend (9)

| # | Chunk | Files | LOC | Focus |
| --- | ------- | ------- | ----- | ------- |
| B1 | Schema, migrations, maintenance | `db/schema.rs`, `db.rs`, `commands/storage.rs`, `commands/backup.rs` | 506 | `user_version` loop, FTS5 triggers, retention, vacuum/backup scheduling, cache eviction |
| B2 | Article queries | `db/articles.rs` | 238 | Insert dedupe, FTS phrase escaping, pagination/sort, read/saved semantics |
| B3 | Feed and tag queries | `db/feeds.rs`, `db/tags.rs` | 283 | Unread counts, delete cascades, Bluesky cursor, tag uniqueness |
| B4 | Connector framework + add/refresh commands | `connectors/mod.rs`, `connectors/add.rs`, `commands/feeds.rs`, `commands/refresh.rs` | 202 | Registry fallthrough, prefetch sharing, idempotent add |
| B5 | RSS and website connectors | `connectors/rss.rs`, `connectors/website.rs` | 584 | Autodiscovery, parse limits, redirects, encoding, scraping |
| B6 | Bluesky connector | `connectors/bluesky.rs` | 331 | Cursor pagination, error handling, article mapping |
| B7 | Command handlers | `commands/crud.rs`, `commands/app.rs` | 380 | Invoke arg shape, mutex scope, settings round-trip |
| B8 | Scrape / content / thumbnails | `commands/scraper.rs`, `commands/content.rs`, `commands/thumbnails.rs` | 539 | SSRF surface, blocking IO in async, WebP cache keys, og:image backfill |
| B9 | Boot, settings, OPML, models | `main.rs`, `lib.rs`, `startup.rs`, `paths.rs`, `settings.rs`, `opml.rs`, `models.rs` | 591 | Setup order, thread spawning, semaphore, dead updater, escaping, log rotation |

## Frontend (9)

| # | Chunk | Files | LOC | Focus |
| --- | ------- | ------- | ----- | ------- |
| F1 | State core + persistence | `store.svelte.ts`, `storeTypes.ts`, `types.ts`, `utils/persistence.ts` | 692 | Composition order, delegation, defaults drift, storage keys |
| F2 | Interaction controllers | `ui.svelte.ts`, `navigation.svelte.ts`, `keyboardNav.svelte.ts`, `shortcuts.svelte.ts` | 567 | Timer lifecycle, pane focus, input guards, remap collisions |
| F3 | Data actions | `articleActions`, `feedActions`, `feedRefresh`, `freshness`, `tags`, `useExpandedFolders`, `utils/{feeds,sets,expandedFolders}` | 675 | Stale-response guard, concurrency, optimistic rollback, unread math, blocked phrases |
| F4 | Navigation UI | `NavPane`, `nav/*` (5 files) | 943 | Drag-and-drop moves, context menus, keyboard activation |
| F5 | Article list + reading | `ArticleListPane`, `article/*` (7 files), `ReadingPane` | 1250 | Pagination/scroll, selection, sanitization, toolbar actions |
| F6 | Settings and modals | `SettingsModal`, `ShortcutsModal`, `AboutModal`, `ConfirmModal`, `Modal`, `ContextMenu` | 1283 | Validation, shortcut recording, focus/Escape |
| F7 | Feed/folder/tag dialogs | `ManageDialog`, `EditFeedDialog`, `NewFolderDialog`, `TagManager`, `TagPopover` | 972 | Form validation, duplicates, tag colors |
| F8 | App shell and utilities | `routes/+layout.svelte`, `routes/+page.svelte`, `ThemeManager`, `StyleInjector`, `Tooltip`, `actions/tooltip`, `utils/{articleContent,dragGhost,favicon,shortcuts,thumbnail}` | 675 | Pane resize, theme flash, CSS injection, tooltip placement |
| F9 | Tests | `src/**/*.test.ts` (13) | 835 | Helper, scheduler and key-routing coverage; shared fixtures in `utils/testFixtures.ts`; Rust tests live inside B chunks |
