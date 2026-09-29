# Changelog
All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

- - -
## v0.23.0 - 2026-09-29

#### Features

- (55ec9a8) use download icon for load full content action - dcog989

- (556312f) add storage management controls - dcog989

- (1d5d142) add Open Latest Log action - dcog989

- (749b591) resolve app directories via Tauri PathResolver - dcog989

#### Bug Fixes

- (580a537) use ui-serif and normal weight for list titles - dcog989

- (8e15bdb) collapse stale expanded folder on startup - dcog989

- (a6822e7) treat zero article retention as "Never" - dcog989

- (90c86a1) keep saved articles when deleting a feed or folder - dcog989

- (f8f3053) prebundle lucide-svelte to stop dev virtual CSS race collapsing layout - dcog989

- (23f1b5d) probe conventional feed paths when HTML discovery fails - dcog989

#### Refactoring

- (c161de0) remove custom-titlebar layout relics - dcog989

- (47b6462) drop no-op gtk titlebar call in setup_window - dcog989

- (e372cdc) replace readabilityrs with dom_smoothie - dcog989

- (a4a30e9) collapse schema migrations into a single baseline - dcog989

- (170c3b5) replace simplelog with flexi_logger for daily log rotation - dcog989

- (1eb7911) replace simplelog with flexi_logger for daily log rotation - dcog989

- - -

## v0.22.13 - 2026-09-24

#### Bug Fixes

- (d58b339) replace spoofed Chrome user agent with FeedMee reader UA - dcog989

- (5e42be3) remove focus outline from article body on keypress - dcog989

- (918bbce) prevent duplicate hero when body image URL is entity-encoded - dcog989

- (5d54e15) prevent stacked keydown listeners and auto-refresh timers - dcog989

- (649f897) clone settings before persisting to avoid aliasing - dcog989

- (fe4a943) restore valid {\@const} syntax mangled by biome --write - dcog989

- (a99baa4) make store the single owner of expandedFolders - dcog989

- (c13b521) guard drag-and-drop payload parsing - dcog989

- (87d9ade) handle reload failure instead of unhandled rejection - dcog989

- (f6b0a4c) catch async clipboard read rejection - dcog989

- (e538bb5) sync unread badge on context-menu read toggle - dcog989

- (768598d) restore unread count when marking read fails - dcog989

- (a91328b) remove recorder keydown listener with matching capture flag - dcog989

- (0ac9853) avoid nesting settings lock inside db lock - dcog989

- (0334c19) preserve config file and warn when settings cannot be parsed - dcog989

- (368fc00) expire date-less articles by insertion time - dcog989

- (6216962) cap image download size before buffering response - dcog989

- (e7d8391) retain MAX_BACKUPS files when rotating backups - dcog989

- (46d8af9) fall back to terminal logging when log file cannot be opened - dcog989

- (0a34322) use unwind panics so article parser guard is effective - dcog989

- (164aa86) prevent UTF-8 panic when truncating post titles - dcog989

- (41918e0) correct inverted sort order in article search - dcog989

- (2ea856e) enable gzip/brotli decompression on the HTTP client - dcog989

- (abc2abb) upgrade low-resolution feed thumbnails via og:image - dcog989

#### Performance Improvements

- (462b994) compare settings without JSON serialization - dcog989

- (cc67236) drop keyed remount for reactive tag icons - dcog989

- (d08d286) compute thumbnail cache key once per card - dcog989

- (942f11c) throttle pane resize to animation frames - dcog989

- (a18bb45) make folder/feed/tag/OPML deletion atomic with transactions - dcog989

- (f0c0df3) add partial index for saved-article queries - dcog989

- (a63d8a4) move blocking maintenance, import, and scrape work off async workers - dcog989

- (56f910f) borrow feed entries instead of cloning the whole feed - dcog989

- (de84e0a) share a single page fetch across feed discovery connectors - dcog989

#### Refactoring

- (e529f7c) use app confirm dialog for tag deletion - dcog989

- (60cfba7) extract pure article-content and feed-selection helpers with tests - dcog989

- (a1a6a20) move modal and layout state into createUI module - dcog989

- (8a255cc) expose narrow store slices via typed accessors - dcog989

- (aafd241) drive empty-state text from a lookup - dcog989

- (c1c95f7) drop redundant two-way binding on AboutModal - dcog989

- (d97a82c) drop unused createTagOps parameter - dcog989

- (c5a432a) simplify domain cache to plain map - dcog989

- (ec9acef) drop unused ShortcutDefinition.id field - dcog989

- (df18b69) compose shared store interfaces to remove duplication - dcog989

- (69cba68) centralize typed localStorage access in persistence module - dcog989

- (72ca64a) centralize Set copy-mutate-reassign in withAdded/withRemoved helpers - dcog989

- (66d4248) reuse runWithConcurrency in refreshAllFeeds - dcog989

- (8c9e0be) centralize folder expand/collapse operations - dcog989

- (5888b35) centralize feed lookups in shared helpers - dcog989

- (1e893be) return named FetchedFeed from discovery API - dcog989

- (3bd19b9) split monolithic db module into schema/feeds/articles/tags - dcog989

- (c14d1e3) move article content extraction out of refresh module - dcog989

- (3a5ce7d) inline registry construction - dcog989

- (02f23c8) drop unused return values and parameter - dcog989

- (60d6b56) remove unused deserialization fields - dcog989

- (facd9db) share page scraping between discovery and refresh - dcog989

- (525b541) centralize feed refresh outcome recording - dcog989

- (fb78489) delegate og:image fetching to shared HTML extractor - dcog989

- (ccd88ba) extract shared OPML rendering used by export and backup - dcog989

- (c63cfcf) deduplicate SQL projections and row mappers - dcog989

- - -

## v0.22.12 - 2026-09-16

- - -

## v0.22.11 - 2026-09-05
- - -

## v0.22.10 - 2026-09-02

#### Bug Fixes

- (c715f85) attach appimage to release and drop raw binary - dcog989
- - -

## v0.22.9 - 2026-09-02

#### Bug Fixes

- (869abe6) upload arch package from hidden .pkg dir - dcog989
- - -

## v0.22.8 - 2026-09-02

#### Bug Fixes

- (7c3d448) install runtime deps in arch container for makepkg check - dcog989
- - -

## v0.22.7 - 2026-09-02

#### Features

- (fa87a75) add folder selector to edit feed dialog - dcog989

#### Bug Fixes

- (52235d2) package prebuilt binary for arch instead of rebuilding in container - dcog989
- - -

## v0.22.6 - 2026-08-30

#### Bug Fixes

- (de785a7) isolate arch container build and install clang - dcog989

- (c1773ea) isolate arch container build and install clang - dcog989
- - -

## v0.22.5 - 2026-08-30

#### Bug Fixes

- (0db8355) mount package.json in arch container for pkgver resolution - dcog989
- - -

## v0.22.4 - 2026-08-29

#### Bug Fixes

- (2aac87e) mount only .pkg dir in arch container to avoid tsconfig conflict - dcog989
- - -

## v0.22.3 - 2026-08-29

#### Bug Fixes

- (eb5d0cb) build arch package on host runner and attach to release - dcog989

- (fa99a9a) install nodejs in arch build container for pkgver - dcog989
- - -

## v0.22.2 - 2026-08-29

#### Bug Fixes

- (f7c0eb9) build arch package on host runner and attach to release - dcog989
- - -

## v0.22.1 - 2026-08-29
- - -

Changelog generated by [cocogitto](https://github.com/cocogitto/cocogitto).