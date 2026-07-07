# Development & Contribution Guide

Welcome to the Nami development guide. This document outlines the standards and workflows for contributing to the project.

## 🛠 Setup

1. **Prerequisites**:
    - Rust (latest stable)
    - Docker (optional, for telemetry stack)
    - Node.js (for WebUI development)
2. **Environment**:
    - Copy `.env.example` to `.env` and add your API keys (e.g., `GOOGLE_API_KEY`).
    - Run `make build` to compile the application and WebUI.

## 🏗 Coding Standards

- **Async First**: Use `tokio` for all asynchronous operations.
- **Error Handling**: Prefer `anyhow` for application-level errors and `AdkError` for tool-specific errors.
- **Documentation**: Every new module or significant component should have a `README.md` or be documented in the `docs/` directory.
- **Idiomatic Rust**: Follow standard Rust conventions. Use `cargo fmt` before committing.

## 🧪 Testing Workflow

Refer to [HARNESS.md](./HARNESS.md) for a deep dive into the testing infrastructure.

- **Unit Tests**: Add tests in `#[cfg(test)]` modules at the bottom of the implementation file.
- **Integration Tests**: Add end-to-end tests in the `tests/` directory.
- **Evaluation**: Use `cargo run -- eval` to verify agent behavior against the `evals.yaml` dataset.

## 📈 Adding a New Tool

1. Create a new directory in `src/tools/`.
2. Implement the `Tool` trait.
3. Register the tool in `src/agent/agent.rs` within the `build_agent` function.
4. Add unit tests to verify the tool's execution logic.

## 📝 Retrospective Reporting

To maintain our "Wiki-First" culture, all complex or non-trivial tasks should include a retrospective report.

1. After completing a task, draft a report in `workspace/wiki/Reports/`.
2. Use the `workspace/wiki/Templates/Task_Retrospective.md` template.
3. Ensure the retrospective includes documented insights and links to updated project documentation.
