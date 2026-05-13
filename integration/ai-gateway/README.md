# AI Gateway (MLflow Deployments)

This directory provides guidance on using **MLflow Deployments Server** as an AI Gateway for Nami. This setup enables high availability through **load balancing** and **fallback strategies** across multiple LLM providers.

## 🏗 Architecture

The AI Gateway acts as a central proxy that manages connections to various LLM backends (OpenAI, Anthropic, Gemini, Local Models, etc.).

```text
Nami App (Rust)
  → MLflow AI Gateway (:5000/endpoints/...)
      → [Strategy: Weighted / Fallback]
          → Primary Provider (e.g., OpenAI GPT-4)
          → Secondary Provider (e.g., Anthropic Claude 3)
```

## 🚀 Configuration

### 1. Create Gateway Configuration (`gateway-config.yaml`)

Create a configuration file to define your endpoints and routing logic:

```yaml
endpoints:
  - name: primary-gpt4
    provider: openai
    config:
      openai_api_key: $OPENAI_API_KEY
      model: gpt-4o

  - name: fallback-claude
    provider: anthropic
    config:
      anthropic_api_key: $ANTHROPIC_API_KEY
      model: claude-3-5-sonnet-20240620

routes:
  - name: chat-resilient
    route_type: llm/v1/chat
    routing_strategy: fallback
    config:
      endpoints:
        - name: primary-gpt4
        - name: fallback-claude
```

### 2. Start the Gateway Server

Run the MLflow deployment server using your configuration:

```bash
mlflow deployments start-server --config-path gateway-config.yaml --port 5000
```

## ⚙️ Nami Integration

To use the AI Gateway in Nami, update your `config.toml` to point to the gateway's OpenAI-compatible endpoint.

### `config.toml`

```toml
[model]
provider = "openai"
# The name of the ROUTE defined in your gateway-config.yaml
model_name = "chat-resilient"
# Environment variable for the key (can be dummy if gateway handles keys)
api_key_env = "OPENAI_API_KEY"
# The base URL of the MLflow Deployment Server
base_url = "http://localhost:5000/endpoints"
```

> **Note**: Ensure that Nami's `openai` provider is configured to respect the `base_url` setting. If using MLflow, the path usually ends with `/endpoints`.

## 💎 Benefits

*   **Load Balancing**: Distribute traffic across multiple API keys or providers to manage rate limits.
*   **Fallback**: Automatically switch to a secondary provider if the primary one is down or returns an error.
*   **Security**: Centralize API key management in the gateway instead of individual client configurations.
*   **Observability**: Monitor all outgoing LLM requests through a single entry point.
