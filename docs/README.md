# Nami Documentation Index

Welcome to the Nami Core developer and system documentation hub. This directory contains comprehensive documentation covering the architecture, modular components, tool development guidelines, and testing frameworks used in the project.

---

## 🗺️ Documentation Directory Map

Explore the different facets of the Nami framework:

### 1. 🏗️ [System Architecture](./ARCHITECTURE.md)
A high-level overview of Nami's design philosophy, data flow, observability setups, and layered component structures.
- **Key topics**: Layered design, asynchronous data flow, OpenTelemetry trace structures, and overall codebase directory mapping.

### 2. 🔌 [Component Reference](./COMPONENTS.md)
The primary code-level reference mapping core modules, shared resources, background loop engines, and interactive interface modes.
- **Key topics**: `AgentRunner`, the `Agent` construction and loading rules, `Reflection` memory architects, background `Scheduler` details, module architecture details for all `src/` subdirectories (`src/agent/`, `src/modes/`, `src/utils/`), and descriptions of all built-in tools.

### 🛠️ 3. [Tool System & MCP Integration](./TOOLS.md)
Detailed architectural guide on Nami's built-in tool system, security sandbox policies, and Model Context Protocol (MCP) integrations.
- **Key topics**: The `Tool` trait, filesystem sandboxing/path neutralization (`.namiignore`), remote/local MCP server configurations, schema sanitization, and prefix mappings.

### 🧪 4. [Testing & Evaluation Harness](./HARNESS.md)
A comprehensive guide on validating Nami's capabilities, logic correctness, and performance using test suits and automated datasets.
- **Key topics**: Unit and integration testing, configuring evaluation datasets in `evals.yaml`, running test suits, quiet logging behaviors, and OpenTelemetry trace analysis.

### 💻 5. [Development & Contribution Guide](./DEVELOPMENT.md)
The official setup guide and contribution standards for building, testing, and developing new features for Nami Core.
- **Key topics**: Prerequisites, build toolchains (`Makefile`), coding guidelines, adding a new tool, and writing task retrospective reports for our wiki-first culture.

---

> [!TIP]
> **Getting Started**: If you are new to developing with Nami Core, we recommend reading the [Development & Contribution Guide](./DEVELOPMENT.md) first to set up your environment, followed by the [System Architecture](./ARCHITECTURE.md) to understand how the system is layered.
