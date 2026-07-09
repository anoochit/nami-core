---
name: system-status
description: Retrieve and report real-time system performance data including CPU usage, memory availability, and disk status. Use this skill whenever the user asks about system health, machine performance, or resource usage — even if they say "how are you running?", "is the server okay?", "what's the CPU at?", "check memory", "am I running out of disk space", or use the "/status" command.
---

# 🖥️ System Status Skill

This skill retrieves real-time telemetry from the host machine using standard command-line tools and presents it as a premium, highly scannable status dashboard.

## Instructions

### 1. Gather Telemetry
Execute the following non-destructive shell commands via your command execution capability to collect host system metrics:

*   **CPU Utilization & Load:** `top -bn1 | grep "Cpu(s)"` or `cat /proc/loadavg`
*   **Memory Footprint:** `free -h` or `cat /proc/meminfo`
*   **Disk Storage:** `df -h /`
*   **Operating System details:** `uname -a` or `cat /etc/os-release`
*   **Developer Toolchains (Versions):** Verify versions of key runtimes such as `cargo --version`, `python --version`, `node --version`, and `git --version`.

### 2. Evaluate & Categorize Resource Health
Analyze the parsed telemetry to assign appropriate status thresholds:
*   🟢 **Normal (0% - 69%):** Everything is operating efficiently.
*   🟡 **Warning (70% - 85%):** High resource consumption. Explain possible causes.
*   🔴 **Critical (86% - 100%):** Severe bottlenecks detected. Flag immediately and suggest mitigation actions (e.g., clearing caches, terminating runaway processes).

### 3. Generate a Premium Visual Status Table
Organize the metrics in an elegant, structured markdown table with status emojis and progress bars:

| Metric | Value / Usage | Status | Health Bar |
| :--- | :--- | :---: | :--- |
| **CPU Load** | 12.4% (Load: 0.45) | 🟢 Normal | ██░░░░░░░░ 12% |
| **Memory** | 4.2 GB / 16.0 GB | 🟢 Normal | ████░░░░░░ 26% |
| **Disk Space** | 85.2 GB / 256.0 GB | 🟢 Normal | ███░░░░░░░ 33% |

### 4. Provide a Consolidated Health Summary
Under the table, include:
1.  **Telemetry Insights:** A 1-2 sentence summary of overall host health.
2.  **Toolchain Versions:** A compact sub-section highlighting available toolchain versions for developer convenience.
    > **Runtimes:** Rust `1.80.1` | Python `3.11.2` | Node.js `20.9.0`
