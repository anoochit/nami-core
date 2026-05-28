# Current Datetime Tool

## Purpose
Provides the Nami agent with the current date, time, and timezone offset information using system clock calculations.

## Architecture & Responsibilities
- **`mod.rs`**: Implements time fetching, UNIX timestamp conversion, day/month string mapping, and timezone offsets without relying on complex external chrono-like library crates.

## Key Entry Points
- `datetime_tools()`: Exports the current datetime toolset (`GetCurrentDatetime`) for registration into the agent's core tools list.

## Tools

### `get_current_datetime`
Gets the current date and time, optionally adjusted for a UTC timezone offset.

#### Arguments
| Parameter | Type | Required | Description |
|---|---|---|---|
| `timezone_offset_hours` | `f64` | No | Optional timezone offset in hours from UTC (e.g. `5.5` for IST, `-5.0` for EST). Defaults to UTC (`0.0`). |

#### Output Structure
Returns a JSON object with the following fields:
```json
{
  "iso8601": "2026-05-28T10:49:34",
  "date": "2026-05-28",
  "time": "10:49:34",
  "day_of_week": "Thursday",
  "day": 28,
  "month": 5,
  "month_name": "May",
  "year": 2026,
  "hour": 10,
  "minute": 49,
  "second": 34,
  "unix_timestamp": 1779965374,
  "timezone": "UTC+7"
}
```

## Maintenance Note
- **Time Synchronization**: Since the tool directly queries `std::time::SystemTime::now()`, ensure that the host system clock is properly synchronized (e.g., via NTP).
- **Date Decomposition**: Uses a custom manual `unix_to_datetime` algorithm shifted to the era beginning 1 Mar 0000 for leap-year calculations.
