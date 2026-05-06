use adk_rust::prelude::*;
use adk_tool::tool;
use schemars::JsonSchema;
use serde_json::{Value, json};
use std::sync::Arc;
use std::time::Instant;
use sysinfo::{Disks, Networks, System};

// ─── Tools ────────────────────────────────────────────────────────────────────

#[derive(serde::Deserialize, JsonSchema)]
struct NoArgs {
    /// Optional reason for requesting system status.
    #[allow(dead_code)]
    reason: Option<String>,
}

/// Retrieves system information including CPU usage, memory stats, disk space, network stats, and latency to google.com.
#[tool]
async fn get_system_status(_args: NoArgs) -> std::result::Result<Value, AdkError> {
    let mut sys = System::new_all();
    sys.refresh_all();

    // CPU information
    let cpu_count = sys.cpus().len();
    let global_cpu_usage = sys.global_cpu_usage();

    // Memory information
    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let total_swap = sys.total_swap();
    let used_swap = sys.used_swap();

    // Disk information
    let disks = Disks::new_with_refreshed_list();
    let mut disk_info = Vec::new();
    for disk in &disks {
        disk_info.push(json!({
            "name": disk.name().to_string_lossy(),
            "mount_point": disk.mount_point().to_string_lossy(),
            "total_space_gb": disk.total_space() as f64 / 1024.0 / 1024.0 / 1024.0,
            "available_space_gb": disk.available_space() as f64 / 1024.0 / 1024.0 / 1024.0,
            "is_removable": disk.is_removable(),
        }));
    }

    // Network information
    let networks = Networks::new_with_refreshed_list();
    let mut network_info = Vec::new();
    for (interface_name, data) in &networks {
        network_info.push(json!({
            "interface": interface_name,
            "received_bytes": data.received(),
            "transmitted_bytes": data.transmitted(),
        }));
    }

    // Measure HTTP latency
    let start = Instant::now();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build();

    let latency_ms = if let Ok(client) = client {
        match client.head("https://www.google.com").send().await {
            Ok(_) => Some(start.elapsed().as_millis()),
            Err(_) => None,
        }
    } else {
        None
    };

    Ok(json!({
        "cpu": {
            "count": cpu_count,
            "global_usage_percent": global_cpu_usage,
        },
        "memory": {
            "total_gb": total_memory as f64 / 1024.0 / 1024.0 / 1024.0,
            "used_gb": used_memory as f64 / 1024.0 / 1024.0 / 1024.0,
            "used_percent": (used_memory as f64 / total_memory as f64) * 100.0,
        },
        "swap": {
            "total_gb": total_swap as f64 / 1024.0 / 1024.0 / 1024.0,
            "used_gb": used_swap as f64 / 1024.0 / 1024.0 / 1024.0,
        },
        "disks": disk_info,
        "networks": network_info,
        "latency_ms": latency_ms,
        "system_name": System::name(),
        "kernel_version": System::kernel_version(),
        "os_version": System::os_version(),
        "host_name": System::host_name(),
    }))
}

// ─── Registration ─────────────────────────────────────────────────────────────

pub fn system_status_tools() -> Vec<Arc<dyn Tool>> {
    vec![Arc::new(GetSystemStatus)]
}
