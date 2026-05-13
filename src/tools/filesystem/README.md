# Filesystem Tool

## Purpose
Provides sandboxed file system operations, allowing the agent to read, write, and list files within the `workspace/` directory safely.

## Architecture & Responsibilities
- **`mod.rs`**: Implements path sandboxing (`sandbox`), ignore policies (`.namiignore`), and CRUD filesystem operations.

## Key Entry Points
- `filesystem_tools()`: Exports filesystem tools for registration.

## Maintenance Note
- Always use the `sandbox` function for path resolution to prevent directory traversal vulnerabilities.
- Ensure the `.namiignore` policy is respected in all new filesystem operations.
