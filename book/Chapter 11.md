# Chapter 11: Security Guardrails 🌊🛡️

Hold up! Before we catch the next big wave of automation, we need to talk about the most important part of the ride: **The Guardrails.** 

Look, Nami Core is powerful. It can move files, call APIs, and execute code faster than a professional surfer on a point break. But with great speed comes the responsibility to not wipe out. We’re not just building an AI; we’re building a trusted first mate. In this chapter, we’re diving into how Nami keeps your "ship" (your system and data) safe from rogue scripts and accidental deletions.

## 1. The "Sandbox" Philosophy: Permission-Based File Management 📂

Nami doesn’t just roam free across your hard drive like a wild current. We implement a **Strict Scoping Policy**. By default, Nami operates within a designated **Workspace**.

### Workspace Isolation
Every task Nami performs is bound to a specific directory path. If Nami tries to `rm -rf /` (heaven forbid!), the core security layer intercepts the call before it even reaches the shell. 

*   **Read-Only zones:** System files and core configurations are strictly off-limits unless explicitly "whitelisted" by the user.
*   **The Sandbox Hook:** Every file operation passes through a validation function.
    ```typescript
    function validatePath(targetPath: string): boolean {
      const resolvedPath = path.resolve(targetPath);
      return resolvedPath.startsWith(process.env.NAMI_WORKSPACE_ROOT);
    }
    ```
*   **Explicit Grants:** If Nami needs to reach outside the workspace, it must trigger a `PermissionRequest` event, requiring a manual "Yes" from the pilot.

## 2. The Vault: Protecting Your Secrets 🔑

In the age of APIs, secrets (API keys, tokens, passwords) are the lifeblood of our integrations. But leaking an `.env` file to a public log is a total wipeout.

### Environment Variable Sanitization
Nami Core uses a "Masking Layer." When Nami processes logs or outputs results to the console, it scans for strings matching known secrets stored in the system.
*   **Automatic Redaction:** If Nami sees a string that matches your `OPENAI_API_KEY`, it’s replaced with `[REDACTED]` before it ever hits a persistent log.
*   **Encrypted Storage:** Secrets aren't stored in plain text within Nami’s memory. We use system-level secret stores (like Keychain or encrypted local DBs) to keep the keys to the kingdom under lock and key.

## 3. Safe Execution: Look Before You Leap 🏄‍♀️

Execution safety is where Nami really shows her tactical side. We don't just "run and pray." We use a multi-stage execution pipeline.

### Pre-Flight Checks (The Dry Run)
Before executing a destructive command (like overwriting a large dataset or deleting folders), Nami performs a **Dry Run**. It simulates the outcome and presents a "Diff" to the user. 
> "Hey! I'm about to change 42 files. Here’s what the first three look like. Should I drop in?"

### The Human-in-the-Loop (HITL) Trigger

For any command categorized as "High-Risk" (Shell scripts, system installs, network configuration changes), Nami enters **Strict Mode**. 
*   **Confirmation Loops:** Nami halts execution and waits for a signed confirmation.
*   **Timeout Safety:** If you don't respond within a set window, Nami aborts the task. Safety first, always!

### Error Rollbacks
If a sequence of commands fails halfway through, Nami doesn't leave your system in a "half-baked" state. Using **Transaction Logging**, Nami tracks every move. If a crash occurs, she can suggest a rollback script to return the workspace to the last "Safe Harbor" (the last committed state).

## 4. The "No-Phone-Home" Policy 🚫📱

Data privacy is the ultimate guardrail. Nami Core is designed to prioritize **local processing**.
*   **Telemetry Control:** You decide what data (if any) leaves the machine.
*   **Local LLM Support:** For the ultimate security-conscious pilot, Nami Core can be hooked into local inference engines (like Ollama), ensuring that your proprietary code never touches a third-party server.

### Summary for the Pilot
Security isn't about slowing down; it's about having the confidence to go fast! With Nami's permission-based file management and secret protection, you can ride the most intense automation waves knowing the guardrails have your back.
