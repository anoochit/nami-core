---
name: system-status
description: Retrieve and report real-time system performance data including CPU usage, memory availability, and disk status. Use this skill whenever the user asks about system health, machine performance, or resource usage — even if they say "how are you running?", "is the server okay?", "what's the CPU at?", "check memory", "am I running out of disk space", or use the "/status" command.
---

# System Status Skill

## Overview

This skill retrieves real-time telemetry from the host machine using standard command-line tools and presents it clearly to the user.

## Instructions

1. Run standard shell commands using your command execution capabilities to gather system metrics:
   - **CPU load & stats**: Run `top -bn1 | grep "Cpu(s)"` or inspect `/proc/loadavg`.
   - **Memory usage**: Run `free -h` or check `/proc/meminfo`.
   - **Disk usage**: Run `df -h /`.
   - **Network interface/IP info**: Run `ip addr` or `ifconfig`.
   - **OS & Kernel information**: Run `uname -a` or view `/etc/os-release`.
   - **Developer Toolchains**: Check versions of common tools like `cargo`, `python3`, `node`, `npm`, `git`, and `docker` using `cmd --version`.

2. Analyze the gathered telemetry:
   - CPU usage percentage
   - Memory usage (used vs total)
   - Disk space status (percentage used/free)

3. Format the results beautifully into a metric summary table:

   | Metric | Value | Status |
   |---|---|---|
   | CPU Usage | <percentage>% | ✅ Normal / ⚠️ Warning |
   | Memory | <used> / <total> | ✅ Normal / ⚠️ Warning |
   | Disk | <used_percentage>% used / <free_space> free | ✅ Normal / ⚠️ Warning |

4. Add a health summary below the table. If any metric is above 80% utilization, flag it with ⚠️ and explain potential slow-downs.

## Examples

**User:** "How is the server doing?"
**Response:** *(Run the bash commands, analyze output, then format output with a table and a friendly summary)*
