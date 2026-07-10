# Nami Agent Performance & Feature Roadmap

A strategic development plan outlining optimizations, architectures, and features to transform Nami into a high-performance, multi-agent AI assistant.

---

## ⚡ Performance Improvements

### 1. Advanced Context Caching (Gemini & Vertex)
To minimize token usage and latency when processing large payloads (PDFs, media folders, long code repositories).
- [ ] Implement automatic TTL-based Gemini Context Caching for static repository contexts.
- [ ] Reuse cached context blocks across consecutive conversation turns instead of re-uploading identical repo matrices.

### 2. Multi-threaded Async Tool Execution
To execute independent tool pipelines in parallel instead of sequentially.
- [ ] Upgrade the `adk-runner` tool loop to support concurrent async execution for disjoint tool calls.
- [ ] Parallelize dependency building and workspace semantic scanning during initial indexing.

### 3. Persistent Local Vector Store & Hybrid Retrieval
Replace simple text-search indexing with a high-performance vector DB (e.g., Qdrant, Milvus, or embedded SQLite-vss).
- [ ] Migrate raw memory scans to a persistent local SQLite-vss embedding index.
- [ ] Implement Hybrid Search combining BM25 keyword matching with dense embedding retrieval for 4x faster file lookups.

---

## 🎨 Premium Feature Roadmap

### 1. Real-Time WebUI Streaming with WebSockets / SSE
To provide instantaneous feedback and low-latency rendering in the WebUI.
- [ ] Replace HTTP poll-based state checking with a full Server-Sent Events (SSE) or WebSocket streaming channel in `src/modes/serve.rs`.
- [ ] Stream model reasoning states, tool execution triggers, and file change logs in real-time.

### 2. Multi-Agent Actor Framework (Autonomous Peer Collaboration)
Let specialists collaborate autonomously on complex engineering pipelines.
- [ ] Develop an actor-based messaging protocol letting `Coder`, `Researcher`, and `Writer` specializations coordinate directly.
- [ ] Introduce a supervisor agent pattern that auto-routes subtasks and validates peer-agent deliverables before reporting to the user.

### 3. Secure Tool Execution Sandbox (Docker / WASM)
Allow safe execution of shell commands, scripts, and compilers without exposing the host environment.
- [ ] Build a lightweight Docker/Podman container runner wrapper for the shell tool.
- [ ] Integrate a sandboxed WebAssembly (WASM) runtime for safe, instant script execution.

---

## 📅 Actionable Implementation checklist

### Phase 1: High-Yield Performance Wins
- [x] **Step 1.1**: Integrate `reqwest` connection pooling and reuse across the model and MCP connections.
- [x] **Step 1.2**: Set up SQLite WAL (Write-Ahead Logging) mode and connection pools inside `adk-session` and memory databases.
- [x] **Step 1.3**: Implement concurrent file reading in `src/tools/filesystem` utilizing `tokio::join!`.

### Phase 2: User Experience & WebUI Upgrades
- [x] **Step 2.1**: Implement a real-time terminal stream visualization panel in the WebUI for background task execution logs.
- [x] **Step 2.2**: Add interactive diff preview blocks inside the WebUI before writing file patches.

### Phase 3: Advanced Architectures
- [x] **Step 3.1**: Build the Multi-Agent Supervisor router letting Nami delegate subtasks concurrently.
- [x] **Step 3.2**: Implement Gemini context cache invalidation handlers for file modification events.
