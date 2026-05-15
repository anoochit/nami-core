# MCP Configuration Guide

Nami supports the Model Context Protocol (MCP) to interact with external tools and data sources. You can configure these connections via the `mcp.json` file.

## Configuration File (`mcp.json`)

The `mcp.json` file should be located in your Nami configuration directory. Below are examples for the two supported transport methods: `stdio` and `SSE` (Stream HTTP).

### 1. Stdio Transport
Use this for local servers that communicate via standard input/output.

```json
{
  "mcpServers": {
    "my-local-server": {
      "command": "node",
      "args": ["/path/to/server/index.js"],
      "env": {
        "KEY": "value"
      }
    }
  }
}
```

### 2. SSE (Stream HTTP) Transport
Use this for remote servers that communicate via Server-Sent Events over HTTP.

```json
{
  "mcpServers": {
    "my-remote-server": {
      "url": "http://localhost:3000/sse"
    }
  }
}
```

Or

```json
{
  "mcpServers": {
    "my-remote-server": {
      "url": "http://localhost:3000/mcp"
    }
  }
}
```

## How to Apply Changes

After creating or modifying `mcp.json`, restart your Nami session or trigger a config reload (if supported by your current environment) to apply the new MCP server configurations.
