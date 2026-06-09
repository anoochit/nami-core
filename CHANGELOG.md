# Changelog

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


