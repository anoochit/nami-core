# Contributing to Nami

Thank you for your interest in contributing to Nami! We are excited to welcome your ideas, code, bug reports, and suggestions.

To ensure a smooth, collaborative experience for everyone, please take a moment to review these guidelines before getting started.

---

## 🛠️ Tech Stack & Structure

Nami is structured as a multi-tier workspace/application combining Rust backend components, Tauri desktop bindings, and frontend user interfaces:

- **Core Backend (`src/`, `Cargo.toml`):** Rust CLI engine, specialists engine, tool orchestrator, and session service.
- **Desktop Shell (`src-tauri/`):** Tauri native desktop application integration.
- **Frontend & Web Interfaces (`webui/`, `website/`):** Web user interface components built with modern frameworks and TailwindCSS.
- **Skills System (`skills/`):** Custom agent skill configurations written in Markdown formats.

---

## 🚀 How to Contribute

### 1. Reporting Bugs
- Search the [Issues](https://github.com/anoochit/nami-core/issues) list to check if your issue has already been reported.
- If not, create a new issue. Please include:
  - A clear and descriptive title.
  - Steps to reproduce the issue.
  - Expected vs. actual behavior.
  - System environment (OS, Rust version, Tauri version, etc.).
  - Relevant log snippets or screenshots.

### 2. Suggesting Features
- We welcome feature requests! Open an issue using the "Feature Request" template.
- Provide a detailed explanation of the proposed feature and why it would benefit the community.

### 3. Submitting Code Changes
1. **Fork the Repository:** Create a personal fork on GitHub.
2. **Create a Feature Branch:** Branch out from `main` (e.g., `git checkout -b feature/awesome-feature` or `git checkout -b bugfix/issue-id`).
3. **Write Code & Tests:** Make your changes, maintaining code quality and ensuring proper testing coverage.
4. **Run Checks:**
   - Run `cargo fmt` to format your Rust code.
   - Run `cargo clippy` to check for common linting/style issues.
   - Run `cargo test` to execute the full unit and integration test suites.
5. **Commit & Push:** Write clean, descriptive commit messages, and push the branch to your fork.
6. **Open a Pull Request (PR):** Target the `main` branch of the upstream repository. Explain your changes clearly in the PR description.

---

## 💻 Rust Coding Standards

To maintain consistency and high code quality across the workspace:

- **Style:** Always run `cargo fmt` before committing.
- **Safety:** Minimize the use of `unsafe` blocks. If unsafe code is required, document it thoroughly with a safety comment.
- **Documentation:** Document all public structs, enums, functions, and traits using standard Rust doc comments (`///`).
- **Error Handling:** Use `anyhow` for high-level errors or user-facing lints, and `thiserror` (or similar custom errors) for internal library APIs.

---

## 📄 Code of Conduct

By contributing to this project, you agree to abide by our [Code of Conduct](file:///home/xavier/namiclaw/CODE_OF_CONDUCT.md). Please report any unacceptable behavior to the project maintainers.

Thank you for contributing to Nami! Your support helps us move faster and deliver richer experiences. 🚀
