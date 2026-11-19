# Changelog
All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

- - -
## v1.24.0 - 2026-11-19

#### Features

- (64f5d28) show when a feed, folder or library is already fresh - dcog989

- (e873a30) mark an article read when leaving it instead of on open - dcog989

- (70d3f50) warn when two commands share a key - dcog989

#### Bug Fixes

- (0a1ad58) load the site's own favicon before falling back to an icon service - dcog989

- (0e23f2d) restore wheel scrolling and held arrow key repeat - dcog989

- (50245fc) run tests in check and keep system install out of package - dcog989

- (ace39c0) stop the reset effect from looping on loadGen - dcog989

- (a80371d) clear remaining svelte-check warnings - dcog989

- (1ea5d0c) only strip a heading that starts with the article title - dcog989

- (db97fa0) let the feed icon fallback survive markup changes - dcog989

- (19a3007) match body images whose file name carries a size suffix - dcog989

- (56ac671) keep the paste menu in text fields and the tooltip on screen - dcog989

- (a269358) let the panes keep keyboard activation and ignore shortcuts behind dialogs - dcog989

- (dc27a22) keep tag edits in place and surface feed and folder save errors - dcog989

- (9a0a975) stop misstating the licence and confirm copies - dcog989

- (642eafe) keep context menus on screen - dcog989

- (f2b1e1a) record keys exactly as they are matched - dcog989

- (b09e536) keep form values valid and clear thumbnails for real - dcog989

- (26712e8) focus dialogs and let only the top one handle Escape - dcog989

- (acff0b0) re-render less, scope search honestly and tag from one popover - dcog989

- (6fa8789) dedupe paged articles and own mark-read in one place - dcog989

- (8cac3f0) keep stale content and stray browser windows out of the reading pane - dcog989

- (6d68504) treat an empty folder as fresh - dcog989

- (23f97d7) correct drag-drop, selection and scroll behaviour - dcog989

- (757ee89) keep arrow keys and startup off the hidden uncategorized folder - dcog989

- (aea95a1) default the refresh debounce to three minutes - dcog989

- (36358f6) let a manual refresh bypass the freshness window - dcog989

- (b27c859) match the database read rule when marking all read - dcog989

- (086fdfa) keep a view after deleting the open feed or folder - dcog989

- (b8cf76a) report a failed Read Later toggle - dcog989

- (7110d5d) bound the blocked-phrase page scan - dcog989

- (2d11ad4) keep the open article when a refresh reloads the list - dcog989

- (b29eda0) apply a feed switch before marking the old feed read - dcog989

- (ec06e45) stop the global handler swallowing arrows in dropdowns - dcog989

- (bbd42b6) disarm the recorder when its dialog closes - dcog989

- (e20119d) match the backend thumbnail default and stop duplicating it - dcog989

- (c9dbbe9) keep keyboard focus in the nav after opening a feed - dcog989

- (3e404f2) recover from a missing saved view or bad stored state - dcog989

- (cf26fbb) set the program name so Wayland reports the app id KWin matches - dcog989

- (3636c15) raise the decode budget to match the dimension guard - dcog989

- (9636420) set the GTK app id so Wayland matches the desktop entry - dcog989

- (4ae4a85) keep the better copy's summary when a low-res duplicate arrives - dcog989

- (14d1b5d) enable reqwest query feature and borrow log spec - dcog989

- (9da8cc6) show the failure dialog from a worker thread - dcog989

- (118f7b7) surface the automatic subscription backups - dcog989

- (2ba616c) write exports from the export command - dcog989

- (7a8df48) keep feed types, nested folders and empty folders - dcog989

- (a1f1670) fall back to defaults for missing keys - dcog989

- (048e478) drop the daily vacuum and run a pass on every launch - dcog989

- (a59f431) keep the database on transient errors and report boot failures - dcog989

- (afff964) bound image decoding and cache memory - dcog989

- (285045a) accept short articles and decode declared charsets - dcog989

- (b86e956) cap page reads and reject navigation links - dcog989

- (22e05ed) validate renamed URLs and normalize OPML feeds - dcog989

- (b23a08c) write settings atomically and apply them live - dcog989

- (406fa6a) catch up pagination and map posts correctly - dcog989

- (586f67c) pick the right feed and decode pages correctly - dcog989

- (1273aa1) skip feeds that are already subscribed - dcog989

- (0953e51) harden feed discovery and make adds atomic - dcog989

- (ca9de4a) group feeds in one query and refuse duplicate feed URLs - dcog989

- (0328de2) reuse the stored tag when the name already exists - dcog989

- (1161c90) look up only fetched URLs and drop orphaned sentinel rows - dcog989

- (21501ec) keep paging when blocked phrases empty a page - dcog989

- (2e24463) resume migrations on upgraded databases and move maintenance off startup - dcog989

- (5855f50) show the window if the frontend never asks for it - dcog989

- (c48b251) refuse feed URLs that point at the local network - dcog989

- (fce890b) drop the phone layout and set a window minimum - dcog989

- (a2826c7) stop saving theme colours on unrelated edits - dcog989

- (e8d99de) reset pkgrel on version bump - dcog989

- (fd03ffa) ship the desktop entry under the app id - dcog989

- (0cbb320) raise the minimum rust version to 1.88 - dcog989

- (4c1ccae) decode gif images - dcog989

- (0bc52a9) declare linux package dependencies for deb and rpm - dcog989

- (2732fbb) add a content security policy that keeps inline styles working - dcog989

#### Performance Improvements

- (274c237) cap image downloads at 10MB - dcog989

#### Refactoring

- (6569545) drop unused base64 dep, use str::trim_prefix - dcog989

- (1d0d339) drop the unreachable input guard and unused clear - dcog989

- (a4e0e8d) share the dialog action-button styles - dcog989

- (a1c0354) let the store own the UI state - dcog989

- (8a94ffe) drop freshness tooltips that can go stale - dcog989

- (7c56056) name the fetch budget for what it bounds - dcog989

- (7a89bdb) send appearance changes as a patch - dcog989

- (38b8b73) move article appearance into its own component - dcog989

- (ff219ad) drop unused store imports - dcog989

- - -

## v1.23.3 - 2026-11-12

#### Bug Fixes

- (8f101b4) stop pane keydown handler swallowing spaces in search input - dcog989

- - -

## v1.23.2 - 2026-11-03

#### Refactoring

- (fcec889) replace rusqlite_migration with user_version loop - dcog989

- - -

## v0.23.1 - 2026-10-17

#### Bug Fixes

- (bef3509) use props type annotation instead of $props type argument - dcog989

- (7b7c1c1) dedupe hero image against CDN-resized body variants - dcog989

- (2beb02d) prevent refresh from re-inserting expired articles - dcog989

- (f62487f) add timestamps to log entries - dcog989

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