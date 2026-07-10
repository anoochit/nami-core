# Changelog

## [0.9.42] - 2026-07-10

### Added

- **Natively Process Multimodal & Document Assets (`analyze_media`)**: Designed and integrated a comprehensive, unified `analyze_media` tool. Nami can now natively process and reason over diverse non-text assets (images like PNG/JPEG, audio like MP3/WAV, videos like MP4/MOV, and PDFs) by extracting raw byte vectors and feeding them as inline multimodal parts directly into vision/multimodal-capable LLM APIs.
- **Dynamic Multimodal Instruction Sets**: Configured the new tool to take optional user-directed inquiry directives (e.g. *"transcribe this podcast"*, *"explain slide 2 of this presentation"*), enabling fine-grained context extraction from audio, video, and documents.
- **Native Audio & Video Generators (`audio_generator` / `video_generator`)**: Implemented powerful content creation tools to generate premium speech/sound effects and video clips from text prompts and starter reference images with fine-grained parameter support (voices, format, speeds, duration, camera motion).
- **Configurable Generation Service Blocks**: Added `[audio_generation]` and `[video_generation]` blocks to `config.toml` structure, allowing users to configure dedicated model names, providers, and key environments for both audio and video models.
- **Interactive Model Setup Prompts**: Integrated Audio and Video configuration prompts inside Nami's interactive CLI setup wizard (`src/modes/init.rs`), enabling step-by-step setup and non-destructive merges.
- **AI Novelist Agent Skill Suite (`novelist`)**: Engineered a comprehensive, modular novelist co-writing skill suite (`skills/novelist/`) with a master orchestrator (`SKILL.md`) and five dedicated domain subskills covering Ideation & World-Building (`IWB-01`), Character Interrogation Engines (`CAI-02`), Structural Plot Architecture (`SPD-03`), Zero-Drafting prose acceleration (`DAZ-04`), and Prose Polishing developmental audits (`PPD-05`). Also bundled detailed references and an author usage guide (`README.md`).
- **Dynamic Binary Self-Upgrade Subcommand (`upgrade`)**: Created a high-reliability `upgrade` subcommand in Nami's terminal entry-point. It automatically checks platform specifications, queries the GitHub Releases API for the latest matching `"nightly-"` snapshot release, maps architecture profiles to standard target triples (e.g. `nami-x86_64-unknown-linux-gnu`, `nami-aarch64-apple-darwin`), asks for user confirmation via interactive prompts, streams the download bytes, and executes a clean in-place hot swap of the active binary. Operates completely independently of setup configurations or missing config environments to ensure administrative recovery is always accessible.

### Fixed

- **AppConfig Fallback Initialization Fields**: Added missing `audio_generation` and `video_generation` fields to the default fallback builders for `AppConfig` in both `src/agent/agent.rs` and `src/main.rs`.
- **Audio & Video Generator Stream Imports**: Added missing `futures::StreamExt` imports to allow correct compilation of LLM stream next() consumption inside the new generators.

## [0.9.41] - 2026-07-09

### Added

- **Exposed Image Generator Schema**: Added `parameters_schema()` to the `ImageGenerator` tool so that models (like Gemini) can discover and use optional parameters like `image_path` (reference image), `aspect_ratio`, and `output_path`.
- **Absolute Image Paths**: Upgraded `ImageGenerator` to return the absolute local filesystem path under `"path"` (and the sandboxed/relative path under `"display_path"`) so external callers can find the output file reliably.
- **WebUI "New Chat" Button**: Added a beautifully styled, premium action button at the top of the left vertical sidebar (and first in the row on mobile). Clicking it instantly spawns a new chat thread, opens the collapsible sidebar panel, and focus-switches to the "Sessions" tab with smooth transitions and micro-animations.

### Fixed

- **Gemini Parallel Tool Calling Error**: Resolved the 400 Bad Request error (*"Please ensure that the number of function response parts is equal to the number of function call parts of the function call turn"*) by removing the redundant post-stream manual tool response flushing in `cli.rs`.
- **Sequential Tool Call Enforcement**: Added a strict `Sequential Execution` directive under the `TOOL STRATEGY` section of the system instruction prompt to force Gemini to issue only one tool call at a time, completely avoiding the underlying `adk-rust` parallel call serialization bug.
- **Robust Image Data Extraction**: Added a `clean_markdown_code_block` helper to the image extraction parser in `ImageGenerator` to automatically strip away Markdown fences and language specifiers if the LLM wraps base64/binary payloads in a code block.

## [0.9.40] - 2026-07-08

### Added

- **Technical Book Production Skill Set (`publishing-studio`)**: Introduced a comprehensive suite of skills and guides for managing the end-to-end technical book publishing lifecycle, including research, writing, editing, marketing, analytics, and a master production guide.
- **Enhanced Weather Skill**: Upgraded the weather skill with multi-day forecasts and robust error handling.
- **Repository Health & Community Standards**: Added standard issue templates for bug reports and feature requests, as well as a Security Policy (`SECURITY.md`), Contributing Guidelines (`CONTRIBUTING.md`), and a Code of Conduct (`CODE_OF_CONDUCT.md`).

## [0.9.39] - 2026-07-08

### Added

- **Branched/Isolated Specialist Workspace Modes**: Introduced `SpecialistSubagentTool` to manage workspace isolation modes (`"inherit"`, `"branch"`, and `"share"`) for specialist subagents. In `"branch"` mode, source files are cloned to an isolated `.subagents` directory and executed inside a localized `NAMI_WORKSPACE` context to prevent cross-contamination.
- **Advanced Shell Execution Security Controls**: Introduced `blocked_commands`, customizable `security_level` (`strict` vs `permissive`), and environment variable sanitization settings inside `[tools.shell]` configuration.
- **Path Traversal Argument Verification**: Implemented strict path traversal validations in the shell tool, preventing relative parent-directory traversal (`..`) or absolute references that attempt to escape authorized sandbox boundaries.

## [0.9.38] - 2026-07-07

### Added

- **Configurable Shell Tool Whitelist**: Introduced structured `[tools.shell]` table with `allowed_commands` configuration inside `config.toml` (and `config.toml.example`). This allows users to easily add custom allowed executables alongside Nami's built-in secure defaults, which are safely cached during initialization.
- **Pre-execution Task Planner (`/plan` command)**: Added a new `/plan` custom command template that guides Nami to formulate a structured implementation/execution plan as a markdown artifact before running any code.

### Fixed

- **AppConfig Initialization Compilation Error**: Added missing `tools` field to the manual `AppConfig` fallback initializer in `src/main.rs` to fix a compilation failure during `cargo check`.

## [0.9.37] - 2026-07-07

### Added

- **Clipboard Support (`/copy` command)**: Added a new built-in `/copy` slash command in CLI mode that copies the last assistant response directly to the system clipboard using the `arboard` crate, featuring proper error handling.
- **Provider & Model Tracking in Statistics**: Upgraded `save_agent_statistic` to receive and track model provider and name. These are written to `.nami/stats.json` for all execution modes (interactive CLI, plan execution steps, direct runs, and REST API server middleware).

### Changed

- **Refactored `nami run` Direct Mode Session Persistence**: Upgraded direct mode (`run_direct`) to share the persistent session storage layer instead of spawning an ephemeral in-memory service, enabling accurate token compaction, usage statistics tracking, and robust logging consistency.

## [0.9.36] - 2026-07-07

### Added

- **Multi-Modal / Reference Image Support (Image-to-Image)**: Upgraded `image_generator` to support a reference image (`image_path` parameter or `image_path` field in `ImagenArgs`).
- **Custom Output Path**: Introduced support for specifying a custom output path (`output_path` parameter or `output_path` field in `ImagenArgs`) to save the generated image directly to that exact sandboxed file path.
- **Intelligent Prompt Auto-Detection Fallback**: Added robust regex-based auto-detection to scan the user prompt text for reference image file names (e.g., `x/cover.png`) and output path instructions (e.g., `save to mock.png` or `as mock.png`), automatically linking and executing them even when structured parameters are not set by the model.

## [0.9.35] - 2026-07-06

### Added

- **Smart Environment Verification**: Optimized fallback flow for `nami init` during re-configuration to ensure all keys and settings default perfectly.
- **Interactive Image Generation Configuration**: Added an interactive Step 6 wizard prompt to `nami init` during initialization, enabling users to easily configure or choose to skip image generation settings (provider, model, and environment variable) with smart defaults.
- **Flexible Re-configuration Mode**: Upgraded `nami init` to automatically detect existing `config.toml` and `.env` files. Users are presented with three choices: re-configure/edit existing settings using smart pre-loaded defaults (with convenient blank password/secret fallbacks to keep existing keys), keep files with a safe non-destructive merge of missing options, or overwrite completely.

## [0.9.34] - 2026-07-06

### Added

- **Fallback `/execute` Command**: Added `/execute` dynamic template command as a default fallback option in the dynamic command registry when not specified in user configurations.
- **Aligned Plan Execution Hint**: Added a clear interactive user tip in the CLI right after successfully completing `/grill` (Interactive Alignment) showing how to immediately execute the newly registered plan.

## [0.9.33] - 2026-07-06

### Removed

- **`nami upgrade` Command**: Completely removed the self-upgrade command, associated code module, and token retrieval functionality.

## [0.9.32] - 2026-07-05

### Added

- **`nami upgrade` Command**: Added a secure, native binary self-upgrade mechanism from GitHub Releases, supporting automatic OS/Architecture detection, user confirmation prompts, and safe NTFS-friendly executable replacement.
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


