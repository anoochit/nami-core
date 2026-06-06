---
name: system-status
description: Retrieve and report real-time system performance data including CPU usage, memory availability, and disk status. Use this skill whenever the user asks about system health, machine performance, or resource usage — even if they say "how are you running?", "is the server okay?", "what's the CPU at?", "check memory", or "am I running out of disk space."
---

# System Status

## Overview

This skill retrieves real-time telemetry from the host machine and presents it clearly. Use the `get_system_status` tool to gather CPU load, memory usage, and storage capacity, then report the results in a readable format.

## Guidelines

- **Be precise**: Report percentages and byte counts converted to GB/MB for readability.
- **Contextual analysis**: If CPU or memory usage is high (>80%), briefly note it as a potential factor in slower response times.
- **Privacy**: Do not report specific process names or user paths unless the user is explicitly troubleshooting a specific issue.
- **Formatting**: Use a table for metric summaries so data is easy to scan at a glance.

## Output Format

Present results in a table like this:

| Metric | Value | Status |
|---|---|---|
| CPU Usage | 12% | ✅ Normal |
| Memory | 16 GB free / 32 GB total | ✅ Normal |
| Disk | 70% free | ✅ Normal |

Add a one-line health summary below the table (e.g., "System is healthy with no resource pressure detected."). If any metric is above 80% utilization, flag it with ⚠️ and add a brief note.

## Examples

**User:** "How is the server doing?"
**Response:** *(Run `get_system_status`, then)* "The system is healthy. CPU is at 12%, 16 GB of 32 GB RAM is free, and disk is 70% free."

**User:** "Machine stats please."
**Response:** *(Provide the metrics table above with current values.)*

**User:** "Is the server slow right now?"
**Response:** *(If CPU >80%)* "CPU usage is elevated at 87%, which may be contributing to slower response times. Memory and disk are both healthy."