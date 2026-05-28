use std::sync::Arc;

use adk_rust::Tool;
use adk_rust::serde::Deserialize;
use adk_tool::{AdkError, tool};
use schemars::JsonSchema;
use serde_json::{Value, json};

/// Arguments for the `get_current_datetime` tool.
#[derive(Deserialize, JsonSchema)]
struct DateTimeArgs {
    /// Optional timezone offset in hours from UTC (e.g. 5.5 for IST, -5.0 for EST).
    /// Defaults to UTC (0.0) if not provided.
    timezone_offset_hours: Option<f64>,
}

/// Get the current date and time, optionally adjusted for a UTC offset.
#[tool]
async fn get_current_datetime(args: DateTimeArgs) -> std::result::Result<Value, AdkError> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let offset_hours = args.timezone_offset_hours.unwrap_or(0.0);
    let offset_secs = (offset_hours * 3600.0) as i64;

    // Get current UTC timestamp in seconds
    let utc_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| AdkError::tool(format!("System time error: {e}")))?
        .as_secs() as i64;

    let local_secs = utc_secs + offset_secs;

    // Manual date/time decomposition from Unix timestamp
    let (year, month, day, hour, minute, second) = unix_to_datetime(local_secs);

    let weekdays = [
        "Thursday",
        "Friday",
        "Saturday",
        "Sunday",
        "Monday",
        "Tuesday",
        "Wednesday",
    ];
    let months = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];

    // Day of week: Jan 1 1970 was a Thursday (index 0)
    let day_of_week = weekdays[(local_secs / 86400).rem_euclid(7) as usize];
    let month_name = months[(month - 1) as usize];

    let tz_label = if offset_hours == 0.0 {
        "UTC".to_string()
    } else if offset_hours > 0.0 {
        format!("UTC+{offset_hours}")
    } else {
        format!("UTC{offset_hours}")
    };

    Ok(json!({
        "iso8601":  format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}"),
        "date":     format!("{year:04}-{month:02}-{day:02}"),
        "time":     format!("{hour:02}:{minute:02}:{second:02}"),
        "day_of_week": day_of_week,
        "day":      day,
        "month":    month,
        "month_name": month_name,
        "year":     year,
        "hour":     hour,
        "minute":   minute,
        "second":   second,
        "unix_timestamp": utc_secs,
        "timezone": tz_label,
    }))
}

/// Convert a Unix timestamp (seconds) into (year, month, day, hour, min, sec).
fn unix_to_datetime(ts: i64) -> (i32, u32, u32, u32, u32, u32) {
    let second = ts.rem_euclid(60) as u32;
    let ts = ts / 60;
    let minute = ts.rem_euclid(60) as u32;
    let ts = ts / 60;
    let hour = ts.rem_euclid(24) as u32;
    let mut days = ts / 24; // days since 1970-01-01

    // Shift epoch to 1 Mar 0000 for easier leap-year math
    days += 719468;
    let era = days.div_euclid(146097);
    let doe = days.rem_euclid(146097); // day of era [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // year of era [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // day of year [0, 365]
    let mp = (5 * doy + 2) / 153; // month prime [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // day [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // month [1, 12]
    let y = if m <= 2 { y + 1 } else { y };

    (y as i32, m as u32, d as u32, hour, minute, second)
}

pub fn datetime_tools() -> Vec<Arc<dyn Tool>> {
    vec![Arc::new(GetCurrentDatetime)]
}

#[cfg(test)]
mod tests;
