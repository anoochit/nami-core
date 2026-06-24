# Changelog

## [0.9.27] - 2026-06-24

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


