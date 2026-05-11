# Modes Module

## Purpose
The `modes` module contains the entry-point implementations for the various ways Nami can be run (CLI, Telegram Bot, HTTP Server, etc.). Each mode acts as an interface layer, bridging the user input to the core agent logic.

## Architecture & Responsibilities
- **`cli.rs`**: Interactive TUI interface with rich formatting and command handling.
- **`bot.rs`**: Telegram integration using the `teloxide` framework.
- **`serve.rs`**: HTTP REST API for external integrations.
- **`browse.rs`**: A specialized serve mode that bundles and serves the WebUI and API endpoints.
- **`init.rs`**: Logic for bootstrapping the project configuration and database.
- **`api.rs`**: RESTful API endpoints for workspace and wiki access.

## Dependencies
- **External**: `axum`, `teloxide`, `tower-http`
- **Internal**: `crate::agent`, `crate::runner`

## Maintenance Note
- When adding a new run mode, ensure the `Commands` enum in `src/main.rs` is updated to include the new variant and registered in the `main` loop.
- Any shared UI helpers should be placed in `ui_utils.rs`.
