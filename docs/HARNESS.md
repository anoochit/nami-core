# Nami Agent Harness

This document describes the testing and evaluation infrastructure for the Nami agent.

## 🧪 Testing

Nami uses a multi-tier testing approach:

### 1. Unit Tests

Localized tests for core logic (sanitization, security, configuration).

```bash
make test  # or cargo test
```

### 2. Integration Tests

Verifies agent lifecycle, component integration, and artifact persistence. These tests are located in the `tests/` directory.

```bash
cargo test --test agent_lifecycle
cargo test --test artifact_integration
```

## 📊 Evaluation Harness (`eval` mode)

The `eval` mode allows you to run the agent against a predefined dataset and verify its responses automatically.

### Running Evaluations

```bash
make eval  # or cargo run -- eval
```

> [!TIP]
> **Quiet Mode Enabled**: When executing evaluations, verbose startup/setup logs and telemetry messages are automatically suppressed to ensure the test progress and summaries are extremely clean and legible on stdout.

### Configuring `evals.yaml`

The evaluation dataset is stored in `evals.yaml` in the project root.

**Format:**

```yaml
- name: "Brief name for the test"
  prompt: "The user prompt to send"
  expected: "The expected response string or pattern"
  match_type: "exact" | "contains" | "regex"
```

* **exact**: Response must match exactly (ignoring leading/trailing whitespace).
* **contains**: Response must contain the expected string.
* **regex**: Response must match the provided regular expression.

## 🔍 Observability

If `OTEL_COLLECTOR` is set in your `.env`, evaluation runs will also emit traces to your observability stack (MLflow), allowing you to debug failed test cases with full trace context.
