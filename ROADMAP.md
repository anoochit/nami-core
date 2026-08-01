# Nami Refactoring Roadmap

## What's Good

| Area | Strength |
| ------ | ---------- |
| **Architecture** | Clean layered design: Config → Agent → Runner → Modes. Trait-based abstraction (`Arc<dyn Agent>`, `Arc<dyn Llm>`) enables flexibility |
| **Security Sandboxing** | `sandbox()` function enforces workspace boundaries with path traversal checks and `.namiignore` support |
| **Shell Injection Mitigation** | Command validation with whitelist/blacklist, blocks backticks and `$()` subshells |
| **Retry & Error Categorization** | `utils/retry.rs` with exponential backoff, `utils/error.rs` with transient vs fatal classification |
| **Tool Ecosystem** | 16+ tool categories covering filesystem, shell, memory, web, knowledge base, image generation |
| **Multi-Mode Support** | 15+ execution modes (CLI, API, Bot, LINE, Serve, Eval, Desktop) sharing common agent core |
| **Documentation** | `docs/ARCHITECTURE.md`, `docs/TOOLS.md`, external book, README is comprehensive |

---

## What's Bad

| Issue | Severity | Location |
| ------- | ---------- | ---------- |
| **`unsafe { std::env::set_var() }` in async context** | 🔴 Critical | `utils/env.rs:9,18,29,30`, `agent/specialists.rs:169,179,183` |
| **API keys in URL query parameters** | 🔴 Critical | `utils/gemini_cache.rs:184-185,208-209` — logged by servers |
| **`expect()` on critical init** | 🟠 High | `main.rs:80,115`, `utils/db.rs:11,12,16`, `utils/client.rs:10` |
| **String-based error matching** | 🟠 High | `utils/error.rs:8-16` — `err_str.contains("rate_limited")` is fragile |
| **Deprecated crates** | 🟠 High | `yaml-rust` (unmaintained), `pretty_env_logger` (unmaintained) |
| **Duplicated code** | 🟠 High | Agent config defaults (2x), runner construction (2x), token calc (3x) |
| **No tests for core modules** | 🟠 High | `runner.rs`, `modes/serve.rs`, `modes/cli.rs`, `agent/mcp.rs` all untested |
| **Blocking in async context** | 🟡 Medium | `agent/agent.rs:224` — `block_on()`, `utils/gemini_cache.rs:304` — sync fs |
| **Regex compiled in loops** | 🟡 Medium | `modes/cli.rs:26`, `tools/km/mod.rs:366,380,389` — should be `OnceLock` |
| **100+ unnecessary `.clone()` calls** | 🟡 Medium | Throughout codebase, especially `modes/cli.rs`, `agent/agent.rs` |

---

## Must Improve Before v1.0

| Priority | Action | Impact |
| ---------- | -------- | -------- |
| **1** | Replace `unsafe set_var()` with `std::sync::RwLock<HashMap>` | Thread safety, correctness |
| **2** | Move API keys from URL params to `Authorization` headers | Security |
| **3** | Replace `expect()` with graceful error handling on init | Reliability |
| **4** | Consolidate duplicated code (config defaults, runner, token calc) | Maintainability |
| **5** | Add tests for `runner.rs`, `serve.rs`, `cli.rs` | Confidence |
| **6** | Replace deprecated crates (`yaml-rust`, `pretty_env_logger`) | Long-term support |
| **7** | Use `OnceLock` for static regex compilation | Performance |
| **8** | Convert string-typed configs to enums (`provider`, `security_level`) | Type safety |

---

## Refactoring Phases

### Phase 1: Security Hardening (Critical)

- [ ] Replace all `unsafe { std::env::set_var() }` with thread-safe alternative
- [ ] Move API keys from URL query parameters to `Authorization` headers
- [ ] Audit and fix any other security concerns

### Phase 2: Error Handling (High)

- [ ] Replace `expect()` calls with graceful error handling
- [ ] Convert string-based error matching to typed enums
- [ ] Add proper error propagation with `thiserror` or similar

### Phase 3: Code Consolidation (High)

- [ ] Extract duplicate agent config defaults to shared function
- [ ] Consolidate runner construction patterns
- [ ] Create shared token estimation utility
- [ ] Extract duplicate regex patterns to static constants

### Phase 4: Dependency Cleanup (High)

- [ ] Replace `yaml-rust` with `serde_yaml` (already in use elsewhere)
- [ ] Replace `pretty_env_logger` with `env_logger` or `tracing`
- [ ] Update outdated dependencies

### Phase 5: Test Coverage (High)

- [ ] Add unit tests for `runner.rs`
- [ ] Add integration tests for `modes/serve.rs`
- [ ] Add integration tests for `modes/cli.rs`
- [ ] Add tests for `agent/mcp.rs` and `agent/reflection.rs`

### Phase 6: Performance (Medium)

- [ ] Replace blocking `block_on()` calls with proper async
- [ ] Use `tokio::fs` instead of `std::fs` in async contexts
- [ ] Implement `OnceLock` for static regex compilation
- [ ] Reduce unnecessary `.clone()` calls

### Phase 7: Type Safety (Medium)

- [ ] Convert `provider: Option<String>` to enum
- [ ] Convert `security_level: Option<String>` to enum
- [ ] Add `#[must_use]` annotations to Result-returning functions
- [ ] Implement `From` traits for error conversion

---

## Version Targets

- **v0.9.52**: Phase 1 (Security Hardening)
- **v0.9.53**: Phase 2 (Error Handling) + Phase 3 (Code Consolidation)
- **v0.9.54**: Phase 4 (Dependency Cleanup)
- **v1.0.0-rc1**: Phase 5 (Test Coverage) + Phase 6 (Performance)
- **v1.0.0**: Phase 7 (Type Safety) + final polish
