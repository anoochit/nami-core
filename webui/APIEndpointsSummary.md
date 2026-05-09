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

## Overview of Endpoint Categories:

*   **Health Check**:
    *   `GET /api/health`: Provides a health check with component status.

*   **Agent Management (Apps)**:
    *   `GET /api/apps`: Lists all available agents.
    *   `GET /api/list-apps`: Offers an `adk-go` compatible app listing.

*   **Session Management**:
    *   `POST /api/sessions`: Creates a new session.
    *   `GET, DELETE /api/sessions/{app_name}/{user_id}/{session_id}`: Allows retrieval or deletion of a specific session.
    *   `GET, POST /api/apps/{app_name}/users/{user_id}/sessions`: Lists or creates sessions for a given user and app.
    *   `GET, POST, DELETE /api/apps/{app_name}/users/{user_id}/sessions/{session_id}`: Comprehensive session management for specific sessions.

*   **Agent Runtime**:
    *   `POST /api/run/{app_name}/{user_id}/{session_id}`: Runs an agent with Server-Sent Events (SSE).
    *   `POST /api/run_sse`: An `adk-go` compatible SSE runtime endpoint.

*   **Artifacts**:
    *   `GET /api/sessions/{app_name}/{user_id}/{session_id}/artifacts`: Lists artifacts associated with a session.
    *   `GET /api/sessions/{app_name}/{user_id}/{session_id}/artifacts/{artifact_name}`: Retrieves a specific artifact.

*   **Debug and Tracing**:
    *   `GET /api/debug/trace/{event_id}`: Retrieves a trace by event ID (admin-only with auth).
    *   `GET /api/debug/trace/session/{session_id}`: Gets all spans for a specific session.
    *   `GET /api/debug/graph/{app_name}/{user_id}/{session_id}/{event_id}`: Provides graph visualization.
    *   `GET /api/apps/{app_name}/users/{user_id}/sessions/{session_id}/events/{event_id}`: Retrieves event data.
    *   `GET /api/apps/{app_name}/users/{user_id}/sessions/{session_id}/events/{event_id}/graph`: Gets graph in a path-style format.
    *   `GET /api/apps/{app_name}/eval_sets`: Stub for retrieving evaluation sets.

*   **UI Protocol**: Endpoints designed for UI interaction and communication:
    *   `GET /api/ui/capabilities`: Provides supported UI protocols and their metadata (versions, features, etc.).
    *   `POST /api/ui/initialize`, `POST /api/ui/message`, `POST /api/ui/update-model-context`: Helper endpoints for additive MCP Apps host-bridge.
    *   `POST /api/ui/notifications/poll`: Polls queued MCP Apps host-bridge notifications.
    *   `POST /api/ui/notifications/resources-list-changed`, `POST /api/ui/notifications/tools-list-changed`: Queues notifications for resource/tool list changes.
    *   `GET /api/ui/resources`: Lists MCP UI resources (`ui://` entries).
    *   `GET /api/ui/resources/read?uri=...`: Reads a registered MCP UI resource.
    *   `POST /api/ui/resources/register`: Registers an MCP UI resource.
    *   **Protocol Negotiation**: Supports negotiation via `uiProtocol`/`ui_protocol` fields in the request body or `x-adk-ui-protocol` header, and `uiTransport`/`ui_transport` fields or `x-adk-ui-transport` header.

*   **A2A Endpoints (Agent-to-Agent)**:
    *   `GET /.well-known/agent.json`: Retrieves the A2A agent card.
    *   `POST /a2a`: The main A2A JSON-RPC endpoint.
    *   `POST /a2a/stream`: Provides A2A streaming capabilities.

*   **Web UI**:
    *   `GET /`: Redirects to the built-in chat interface.
    *   `GET /ui/`: Serves the built-in chat interface.
    *   `GET /ui/assets/config/runtime-config.json`: Provides runtime configuration for the UI.
    *   `GET /ui/{*path}`: Serves static UI assets.