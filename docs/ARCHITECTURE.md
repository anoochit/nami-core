# System Architecture: Nami Core

Nami is a modular, multi-modal AI agent framework designed for terminal-first workflows, persistent memory, and complex tool usage. It is built in Rust using the `adk-rust` (Agent Development Kit) ecosystem.

## 🏗 Layered Design

Nami follows a layered architecture that separates the user interface (modes) from the core agent logic and external tools.

```text
┌───────────────────────────────────────────────────────────┐
│                      Interface Layer                      │
│   (CLI, Bot, Serve, LINE, Scheduler)                      │
├───────────────────────────────────────────────────────────┤
│                     Orchestration Layer                   │
│         (AgentRunner, Session Management, Retries)        │
├───────────────────────────────────────────────────────────┤
│                        Agent Layer                        │
│         (LlmAgent, Persona, Skills, Specialists)          │
├───────────────────────────────────────────────────────────┤
│                 Services & Resources Layer                │
│ (Sessions, Memory/Vector, Reflection, MCP, FS, Telemetry) │
└───────────────────────────────────────────────────────────┘
```

## 🔄 Data Flow

1.  **Input**: User input enters through one of the Interface modes (e.g., CLI prompt, LINE webhook).
2.  **Orchestration**: The `AgentRunner` retrieves the relevant session state and initializes a `Runner` from the `adk-runner` crate.
3.  **Processing**: The `Runner` sends the context and input to the configured LLM provider.
4.  **Tool Execution**: If the LLM requests a tool (e.g., `list_files`), the `Runner` executes the tool locally (applying sandboxing) and returns the result to the LLM.
5.  **Output**: The final streaming response is processed by the `AgentRunner` and delivered back to the interface layer.

## 📊 Observability & Telemetry

Nami integrates with **OpenTelemetry** to provide full transparency into agent operations.

-   **Traces**: Every interaction, tool call, and LLM request is traced.
-   **Metrics**: Latency, token usage, and error rates are captured.
-   **Export**: Data is exported via OTLP (gRPC) to a collector. A default observability stack using MLflow is provided in `integration/telemetry`.

## 📁 Project Structure

-   `src/lib.rs`: The library entry point, exposing core modules for testing and external use.
-   `src/main.rs`: The binary entry point, handling CLI parsing and service initialization.
-   `src/agent/`: Core agent logic, persona loading, and reflection service.
-   `src/modes/`: Implementations for different interaction patterns.
-   `src/tools/`: The built-in tool library.
-   `src/utils/`: Shared utilities (sandboxing, security, error handling).
-   `workspace/`: The agent's persistent sandbox for files and documentation.
