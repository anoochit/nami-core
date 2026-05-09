---
title: "ADK-Rust API Endpoints Summary"
description: "Summary of API Endpoints for adk-server, part of ADK-Rust, covering Health, Apps, Sessions, Runtime, Artifacts, Debug, UI Protocol, A2A, and Web UI endpoints."
date: 2024-07-31
tags:
  - adk-rust
  - api
  - endpoints
  - rust
---
# ADK-Rust API Endpoints Summary

`adk-server` provides a comprehensive set of HTTP API Endpoints for interacting with Rust Agent Development Kit (ADK-Rust) agents, managing sessions, artifacts, and supporting UI protocols.

## API Endpoints

### Health

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/health` | GET | Health check with component status |

### Apps

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/apps` | GET | List available agents |
| `/api/list-apps` | GET | adk-go compatible app listing |

### Sessions

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/sessions` | POST | Create session |
| `/api/sessions/{app_name}/{user_id}/{session_id}` | GET, DELETE | Get or delete session |
| `/api/apps/{app_name}/users/{user_id}/sessions` | GET, POST | List or create sessions |
| `/api/apps/{app_name}/users/{user_id}/sessions/{session_id}` | GET, POST, DELETE | Get, create, or delete session |

### Runtime

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/run/{app_name}/{user_id}/{session_id}` | POST | Run agent with SSE |
| `/api/run_sse` | POST | adk-go compatible SSE runtime |

### Artifacts

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/sessions/{app_name}/{user_id}/{session_id}/artifacts` | GET | List artifacts for a session |
| `/api/sessions/{app_name}/{user_id}/{session_id}/artifacts/{artifact_name}` | GET | Get a specific artifact |

### Debug and Tracing

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/debug/trace/{event_id}` | GET | Get trace by event ID (admin only when auth configured) |
| `/api/debug/trace/session/{session_id}` | GET | Get all spans for a session |
| `/api/debug/graph/{app_name}/{user_id}/{session_id}/{event_id}` | GET | Get graph visualization |
| `/api/apps/{app_name}/users/{user_id}/sessions/{session_id}/events/{event_id}` | GET | Get event data |
| `/api/apps/{app_name}/users/{user_id}/sessions/{session_id}/events/{event_id}/graph` | GET | Get graph (path-style) |
| `/api/apps/{app_name}/eval_sets` | GET | Get evaluation sets (stub) |

### UI Protocol

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/ui/capabilities` | GET | Supported UI protocols plus capability metadata (`versions`, `features`, `implementationTier`, `specTrack`, `summary`, `limitations`) |
| `/api/ui/initialize` | POST | Additive MCP Apps host-bridge initialize helper (direct body or JSON-RPC-like envelope) |
| `/api/ui/message` | POST | Additive MCP Apps host-bridge message helper |
| `/api/ui/update-model-context` | POST | Additive MCP Apps host-bridge model-context helper |
| `/api/ui/notifications/poll` | POST | Poll queued MCP Apps host-bridge notifications |
| `/api/ui/notifications/resources-list-changed` | POST | Queue an MCP Apps resource-list-changed notification |
| `/api/ui/notifications/tools-list-changed` | POST | Queue an MCP Apps tool-list-changed notification |
| `/api/ui/resources` | GET | List MCP UI resources (`ui://` entries) |
| `/api/ui/resources/read?uri=...` | GET | Read a registered MCP UI resource |
| `/api/ui/resources/register` | POST | Register an MCP UI resource (validated `ui://` + mime/meta) |

Runtime endpoints support protocol negotiation via:
- request body field `uiProtocol` / `ui_protocol`
- header `x-adk-ui-protocol` (takes precedence)
- request body field `uiTransport` / `ui_transport`
- header `x-adk-ui-transport` (takes precedence)

Supported runtime profile values: `adk_ui` (default), `a2ui`, `ag_ui`, `mcp_apps`.

Current support is intentionally tiered:
- `a2ui` is a draft-aligned hybrid subset exposed through protocol-aware UI tool payloads.
- `ag_ui` is a hybrid subset: the default stream remains the generic ADK wrapper, but clients can opt into `protocol_native` transport plus AG-UI run input fields on `/api/run_sse`.
- `mcp_apps` is a compatibility subset with `ui://` resource registration plus additive `initialize` / `message` / `update-model-context` bridge helpers, notification polling, list-changed host flows, and runtime request fields, not a full browser `postMessage` host bridge yet.

Runtime transport values:
- `legacy_wrapper` (default) preserves the existing generic ADK SSE envelope.
- `protocol_native` is currently available for `ag_ui` only.

Use `/api/ui/capabilities` instead of assuming full upstream protocol parity.

For MCP Apps tool responses, `adk-server::ui_types` now exposes a canonical additive helper:
- `McpUiBridgeSnapshot` for typed host/app bridge state that can be promoted into tool responses
- `McpUiToolResult` for the shared tool-result envelope
- `McpUiToolResultBridge` for typed bridge metadata (`protocolVersion`, `structuredContent`, `hostInfo`, `hostCapabilities`, `hostContext`, `appInfo`, `appCapabilities`, `initialized`)

Use `McpUiBridgeSnapshot::build_tool_result(...)` as the preferred constructor path when promoting framework bridge state into an MCP Apps tool response. `resourceUri` and inline `html` fallbacks remain available for compatibility-oriented hosts.

For embedded-host mappings, the additive HTTP bridge corresponds to MCP Apps host/app methods as follows:
- `ui/initialize` -> `/api/ui/initialize`
- `ui/message` -> `/api/ui/message`
- `ui/update-model-context` -> `/api/ui/update-model-context`
- `notifications/resources/list_changed` -> `/api/ui/notifications/resources-list-changed`
- `notifications/tools/list_changed` -> `/api/ui/notifications/tools-list-changed`
- queued host notifications -> `/api/ui/notifications/poll`

### A2A Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/.well-known/agent.json` | GET | A2A agent card |
| `/a2a` | POST | A2A JSON-RPC |
| `/a2a/stream` | POST | A2A streaming |

### Web UI

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | Redirect to `/ui/` |
| `/ui/` | GET | Built-in chat interface |
| `/ui/assets/config/runtime-config.json` | GET | Runtime configuration |
| `/ui/{*path}` | GET | Static UI assets |

## Security

The server applies the following security layers automatically:

- CORS (configurable allowed origins)
- Request body size limits (default 10MB)
- Request timeouts (default 30s)
- `X-Content-Type-Options: nosniff`
- `X-Frame-Options: DENY`
- `X-XSS-Protection: 1; mode=block`
- Request ID tracking via `x-request-id` header
- User ID authorization on session/artifact/debug endpoints when auth is configured

## Features

- Axum-based async HTTP server
- CORS support with configurable origins
- Embedded web UI assets
- Multi-agent routing via `AgentLoader`
- Health checks with component status
- OpenTelemetry trace integration
- Auth middleware bridge for identity propagation
- Artifact storage and retrieval
- A2A v1.0.0 protocol with JSON-RPC 2.0 (all 11 operations, idempotency, multi-turn, push auth)