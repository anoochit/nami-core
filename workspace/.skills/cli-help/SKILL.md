---
name: cli-help
description: Provides a standardized help interface for the Nami CLI, detailing available commands, flags, and usage patterns.
---

# cli-help

This skill provides a centralized help interface for the Nami CLI.

## Overview
The Nami CLI offers comprehensive support for managing your workflow. Access the help information by using the standard command:

```bash
nami help
```

## Available Commands
- `nami init`: Initialize a configuration.
- `nami bot`: Start the interactive Telegram Bot.
- `nami serve`: Start the local server for API interactions.
- `nami cli`: Local interactive terminal agent with rich TUI.
- `nami run "<prompt>"`: Execute a single prompt directly from the CLI.
- `nami "<prompt>"`: Execute a single prompt directly from the CLI.
- `nami help`: Display detailed usage instructions for specific commands.

## Troubleshooting
- If commands are not found, ensure the Nami CLI is installed and in your system PATH.
- For issues with command execution, verify your environment configuration and ensure you are within a valid Nami workspace.
