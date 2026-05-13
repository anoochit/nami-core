# Utils Module

## Purpose
The `utils` module provides shared, low-level utilities and helper functions used across the codebase, focusing on system infrastructure, security enforcement, and path management.

## Architecture & Responsibilities
- **`mod.rs`**: Core workspace path management (`get_workspace_dir`, `sandbox`) and wiki directory helpers.
- **`ignore.rs`**: Implementation of the `.namiignore` policy system, ensuring security compliance for file-system tools.

## Key Entry Points
- `get_workspace_dir()`: Retrieves the secure, sandboxed project workspace root.
- `sandbox(&str)`: Path validation utility that prevents directory traversal and enforces ignore policies.
- `get_wiki_dir()`: Standardized path getter for wiki-related operations.

## Maintenance Note
- Keep this module lean and focused on project-wide infrastructure.
- Any new security policies or path-sanitization rules should be implemented here to ensure global adherence.
