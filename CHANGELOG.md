# Changelog

## [0.9.33] - 2026-07-06

### Removed

- **`nami upgrade` Command**: Completely removed the self-upgrade command, associated code module, and token retrieval functionality.

## [0.9.32] - 2026-07-05

### Added

- **`nami version` Command**: Added a quiet, direct subcommand to display the current Nami version cleanly to standard output.

## [0.9.31] - 2026-07-05

### Added

- **Token-Safe Native Filesystem Tools**: Upgraded filesystem tools (`read_file`, `write_to_file`, `replace_text`, `grep_search`, `glob_find`) to be native, cross-platform (robust on Windows using `walkdir` and `globset`), and token-safe via paged reads, line-bound edits, unique-match validations, and path boundary enforcement.
- **`/pev` Default Slash Command**: Added `/pev` slash command to all CLI modes and registries, including default autocomplete suggestions and configuration templates.
- **Safe Incremental Merges in `nami init`**: Upgraded `nami init` logic to perform non-destructive merges on `config.toml` (TOML table merge) and `.env` (key appends) while preserving existing markdown instructions (e.g., `rules.md`).

## [0.9.30] - 2026-07-01

- **Strip Autocomplete `@` Prefix (WebUI)**: Added `stripAtPrefixes` client-side clean utility to automatically strip leading `@` prefixes from file/wiki mentions (e.g. converting `@test_file.txt` to `test_file.txt`), resolving a file-reading error where backend tools couldn't find the file.

## [0.9.29] - 2026-07-01

### Added

- **Explicit Thread Timestamps (WebUI)**: Added a `createdAt` timestamp property to the `Thread` interface to store the creation datetime explicitly and cleanly.
- **Formatted Thread Datetime in Sidebar**: Replaced the raw sidebar thread ID display under Active Chats with a beautifully formatted localized datetime (e.g., `Jul 1, 08:34`), utilizing a fallback to `thread.id` for backward-compatible parsing and a hover tooltip showing the full timestamp.
- **Text & Base64 Multimedia Support for Filesystem Tool**: Upgraded the `read_file` tool to support explicit and automatic encoding detection (`text` vs. `base64`) along with mime-type guessing using `mime_guess` (returning `"encoding"` and `"mime_type"` fields), allowing seamless reading of plain text as well as multimedia files (images, audio, video).
- **Persistent Multi-Thread WebUI Chat**: Implemented client-side local-storage-backed multi-thread conversation management in the WebUI. Users can now create, delete, and switch between separate chats. Thread titles are automatically generated from the first 30 characters of the initial user prompt.

### Changed

- **Standardized Session ID Display**: Fixed an issue where the displayed thread ID under "Recent Sessions / Active Chats" was different from the backend/header session UUID. It now correctly renders the thread's backend `sessionId` UUID instead of the local client-side `thread.id` timestamp.
- **Upgraded ADK Crate Dependencies**: Bumped all core `adk-*` dependencies (`adk-core`, `adk-memory`, `adk-runner`, `adk-rust`, `adk-session`, `adk-telemetry`, `adk-tool`) to `1.0.0` from `0.9.1` in `Cargo.toml`.

## [0.9.28] - 2026-06-26

### Added

- **TODOs Management in WebUI**: Added a beautifully integrated, fully featured TODOs configuration and management panel inside the WebUI sidebar. Users can now view pending and completed tasks, check/uncheck items, add new items with a quick form, and delete items from their workspace tasks instantly.
- **TODOs Axum REST API Endpoints**: Created Axiom-based supporting REST API endpoints (`GET /api/todos`, `POST /api/todos`, `POST /api/todos/{id}/toggle`, and `DELETE /api/todos/{id}`) in the Rust backend to read and write `todos.json` files, fully integrated with standard tracing instrumentation.

## [0.9.27] - 2026-06-24

### Added

- **Scheduler Configuration in WebUI**: Added a fully featured background task scheduler configuration panel directly in the WebUI sidebar. Users can now view active schedules, register repeating tasks using custom Cron expressions, utilize quick preset helpers, delete schedules, and toggle active status in real-time.
- **Scheduler Axum REST API Endpoints**: Created full supporting REST API endpoints (`GET /api/scheduler`, `POST /api/scheduler/add`, `DELETE /api/scheduler/{id}`, and `POST /api/scheduler/{id}/toggle`) in the Rust backend to manage `scheduler.json` files and sync instantly with the scheduler loop.

### Changed

- **Filter Out Empty Sessions**: Updated the session listing query in [api.rs](file:///D:/Projects/AIProject/namiclaw/src/modes/api.rs) to only return sessions that contain at least one message/event. This prevents listing recent sessions with empty conversations in the WebUI sidebar.

## [0.9.27] - 2026-06-09

### Added

- **Automatic Workspace Detection & Activation**: Upgraded workspace resolution in [get_workspace_dir](file:///home/xavier/namiclaw/src/utils/mod.rs#L222) to automatically set the active workspace and register the directory to the global workspaces list in `config.toml` whenever Nami is run in a directory.
- **Wiki Search Fallback Interactive Prompts**: Updated the tool execution strategy in [format_persona](file:///home/xavier/namiclaw/src/agent/agent.rs#L543) to prompt the agent to explicitly ask the user whether it should search the project workspace files when a wiki page/information is not found.
- **Responsive Icon Navigation Sidebar (WebUI)**: Replaced the tab-based sidebar in [App.tsx](file:///home/xavier/namiclaw/webui/src/App.tsx) with a light vertical icon sidebar on desktop, which responsively transforms into a horizontal bottom navigation bar on mobile.
- **Mobile Collapsible Drawer Overlay (WebUI)**: Added a mobile drawer overlay for the sidebar panel, sliding up cleanly from the bottom bar when open and collapsing to `translate-y-full` when closed.

## [0.9.27] - 2026-06-08

### Added

- **Wiki CRUD Support**: Added the `delete_wiki_page` tool for complete page deletion capabilities from the Wiki vault, completing full CRUD support.
- **Wiki Grep & Glob Support**: Integrated robust recursive glob pattern matching with the new `glob_find_wiki` tool using the `globset` crate, complementing existing regex (Grep) search capabilities in `search_wiki`.
- **Filesystem Delete Support**: Added the `delete_file` tool to programmatically delete files within the workspace boundary.
- **FilePreview Syntax Highlighting Upgrade**: Updated the `FilePreview` component in the WebUI to support proper syntax highlighting for JSON, TypeScript, TSX, JavaScript, JSX, Rust, Python, and CSS by registering PrismLight language modules and dynamically mapping file extensions to language names.
- **Serve Mode Dynamic Workspace API**: Introduced dedicated REST endpoints (`GET /api/workspaces`, `POST /api/workspaces/add`, and `POST /api/workspaces/select`) to enable programmatic listing, adding, and selecting of workspaces in server mode, bringing feature parity with CLI mode.
- **Dynamic Active Workspace Autocomplete**: Replaced the hardcoded autocomplete in CLI mode with a dynamic resolver that reads registered workspaces from `config.toml` and falls back to the current directory, supporting absolute and relative paths under the `@` prefix.
- **CLI Mode Tab-to-Complete Keybinding**: Programmatically bound the `Tab` key to the standard `Complete` command inside the `rustyline` helper, guaranteeing interactive completions trigger instantly under all terminal sessions.
- **Prefixless Filename Autocomplete**: Upgraded the autocomplete engine to match plain typed words as workspace filenames even when the `@` prefix is omitted.
- **Automatic Prefix Insertion**: Upon selecting a prefixless match, the `@` symbol is automatically prepended to the completed filename (e.g. typing `src/mo` completes to `@src/modes/cli.rs`) to ensure file references are cleanly identified and parsed within interactive user prompts.

### Changed

- **Autocomplete Performance Optimization**: Pruned common heavy development and build directories (including `.git`, `node_modules`, `target`, `dist`, `.venv`, and `build`) using `WalkDir::filter_entry` in the autocomplete resolver, making path suggestion retrieval instantaneous.
- **Strict Workspace Boundary for Autocomplete**: Enforced that the autocomplete walks strictly within the active workspace root, stripping out leading absolute path prefixes and resolving all completed entries relative to the workspace boundary for consistency and sandbox integrity.


