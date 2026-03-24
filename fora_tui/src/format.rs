use chrono::{DateTime, Utc};

/// Formats the runtime between `start` and `end` as a human-readable string.
///
/// - Both present → completed duration (e.g. `"2h 15m"`)
/// - Only `start` → live elapsed with hourglass (e.g. `"1h 23m ⏳"`)
/// - Missing `start` → `"—"`
pub fn format_runtime(start: Option<DateTime<Utc>>, end: Option<DateTime<Utc>>) -> String {
    let start = match start {
        Some(s) => s,
        None => return "—".to_string(),
    };

    let reference = end.unwrap_or_else(Utc::now);
    let duration = reference.signed_duration_since(start);

    // Negative or zero duration (clock skew, etc.)
    if duration.num_seconds() <= 0 {
        return if end.is_none() {
            "0s ⏳".to_string()
        } else {
            "0s".to_string()
        };
    }

    let total_secs = duration.num_seconds();
    let formatted = format_duration_secs(total_secs);

    if end.is_none() {
        format!("{} ⏳", formatted)
    } else {
        formatted
    }
}

/// Formats a number of seconds into a compact human-readable string.
///
/// - `< 60s` → `"42s"`
/// - `< 1h`  → `"45m 30s"`
/// - `< 1d`  → `"2h 15m"`
/// - `≥ 1d`  → `"3d 1h"`
fn format_duration_secs(total_secs: i64) -> String {
    let days = total_secs / 86_400;
    let hours = (total_secs % 86_400) / 3_600;
    let minutes = (total_secs % 3_600) / 60;
    let seconds = total_secs % 60;

    if days > 0 {
        format!("{}d {}h", days, hours)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_completed_job_seconds() {
        let start = Utc::now() - Duration::seconds(42);
        let end = Utc::now();
        assert_eq!(format_runtime(Some(start), Some(end)), "42s");
    }

    #[test]
    fn test_completed_job_minutes() {
        let end = Utc::now();
        let start = end - Duration::seconds(45 * 60 + 30);
        assert_eq!(format_runtime(Some(start), Some(end)), "45m 30s");
    }

    #[test]
    fn test_completed_job_hours() {
        let end = Utc::now();
        let start = end - Duration::seconds(2 * 3600 + 15 * 60);
        assert_eq!(format_runtime(Some(start), Some(end)), "2h 15m");
    }

    #[test]
    fn test_completed_job_days() {
        let end = Utc::now();
        let start = end - Duration::seconds(3 * 86400 + 3600);
        assert_eq!(format_runtime(Some(start), Some(end)), "3d 1h");
    }

    #[test]
    fn test_running_job_has_hourglass() {
        let start = Utc::now() - Duration::seconds(90);
        let result = format_runtime(Some(start), None);
        assert!(result.contains("⏳"));
        assert!(result.contains("1m"));
    }

    #[test]
    fn test_no_start_time() {
        assert_eq!(format_runtime(None, None), "—");
        assert_eq!(format_runtime(None, Some(Utc::now())), "—");
    }
}
