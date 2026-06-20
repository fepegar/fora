//! Lightweight line-by-line highlighter for plain-text log files.
//!
//! Designed for Azure ML `user_logs/std_log*.txt` and friends, which
//! mostly carry framework log lines (`INFO`, `WARN`, `ERROR`) plus
//! occasional Python tracebacks. syntect doesn't ship a "Log" or
//! "Python Traceback" syntax in its default set, so we do the coloring
//! ourselves with cheap per-line regexes.
//!
//! Output is a `Vec<Line<'static>>` of pre-styled spans, ready to feed
//! into `Paragraph::new` and cache.

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use regex::Regex;
use std::sync::OnceLock;

static FILE_LINE_RE: OnceLock<Regex> = OnceLock::new();
static TIMESTAMP_RE: OnceLock<Regex> = OnceLock::new();
static LEVEL_RE: OnceLock<Regex> = OnceLock::new();
static EXCEPTION_TAIL_RE: OnceLock<Regex> = OnceLock::new();
static TRACEBACK_HEADER_RE: OnceLock<Regex> = OnceLock::new();

fn file_line_re() -> &'static Regex {
    FILE_LINE_RE.get_or_init(|| {
        // `  File "path", line 123, in funcname`
        Regex::new(
            r#"^(?P<indent>\s+)File "(?P<path>[^"]+)", line (?P<line>\d+), in (?P<func>.+)$"#,
        )
        .unwrap()
    })
}

fn timestamp_re() -> &'static Regex {
    TIMESTAMP_RE.get_or_init(|| {
        // Common log timestamps:
        //   2025-05-31 14:23:01,234   2025-05-31T14:23:01.234Z   [2025-05-31 14:23:01]
        Regex::new(
            r"^(?P<ts>\[?\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}(?:[.,]\d+)?(?:Z|[+-]\d{2}:?\d{2})?\]?)",
        )
        .unwrap()
    })
}

fn level_re() -> &'static Regex {
    LEVEL_RE.get_or_init(|| {
        // Match a level word as its own token, optionally bracketed.
        // Word boundaries on both sides avoid matching e.g. `INFOrmation`.
        Regex::new(r"\b(TRACE|DEBUG|INFO|NOTICE|WARN|WARNING|ERROR|ERR|CRITICAL|FATAL)\b").unwrap()
    })
}

fn exception_tail_re() -> &'static Regex {
    EXCEPTION_TAIL_RE.get_or_init(|| {
        // Trailing exception line: `RuntimeError: something went wrong`
        // (also matches qualified exception names like `torch.cuda.OutOfMemoryError`)
        Regex::new(r"^(?P<exc>[A-Z][A-Za-z0-9_.]+(?:Error|Exception|Warning|Exit|Interrupt)):\s*(?P<msg>.*)$")
            .unwrap()
    })
}

fn traceback_header_re() -> &'static Regex {
    TRACEBACK_HEADER_RE
        .get_or_init(|| Regex::new(r"^(?P<hdr>Traceback \(most recent call last\):)\s*$").unwrap())
}

/// True if the body contains at least one Python traceback marker.
pub fn looks_like_python_traceback(body: &str) -> bool {
    body.lines()
        .any(|l| traceback_header_re().is_match(l) || file_line_re().is_match(l))
}

/// Highlight a plain-text log body line-by-line. Returns owned
/// `Line<'static>` rows ready for `Paragraph::new`.
pub fn highlight_log_text(body: &str) -> Vec<Line<'static>> {
    let mut out: Vec<Line<'static>> = Vec::with_capacity(body.lines().count());
    for raw in body.lines() {
        out.push(highlight_line(raw));
    }
    out
}

fn highlight_line(line: &str) -> Line<'static> {
    // 1) Python traceback header — bold red on the full line.
    if let Some(c) = traceback_header_re().captures(line) {
        return Line::from(Span::styled(
            c["hdr"].to_string(),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ));
    }

    // 2) `  File "path", line N, in func`
    if let Some(c) = file_line_re().captures(line) {
        let indent = c["indent"].to_string();
        let path = c["path"].to_string();
        let line_no = c["line"].to_string();
        let func = c["func"].to_string();
        return Line::from(vec![
            Span::raw(indent),
            Span::styled("File ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("\"{}\"", path), Style::default().fg(Color::Cyan)),
            Span::styled(", line ", Style::default().fg(Color::DarkGray)),
            Span::styled(line_no, Style::default().fg(Color::Yellow)),
            Span::styled(", in ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                func,
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
    }

    // 3) Trailing exception line: `RuntimeError: msg`.
    //    Only color when the line is *not* indented; indented "Error:"
    //    text inside log messages would otherwise be over-highlighted.
    if !line.starts_with([' ', '\t']) {
        if let Some(c) = exception_tail_re().captures(line) {
            return Line::from(vec![
                Span::styled(
                    c["exc"].to_string(),
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled(": ", Style::default().fg(Color::DarkGray)),
                Span::styled(c["msg"].to_string(), Style::default().fg(Color::Red)),
            ]);
        }
    }

    // 4) Generic log line: optional timestamp + optional level + rest.
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut rest = line;

    if let Some(c) = timestamp_re().captures(rest) {
        let mat = c.name("ts").unwrap();
        let ts = mat.as_str().to_string();
        spans.push(Span::styled(ts, Style::default().fg(Color::DarkGray)));
        rest = &rest[mat.end()..];
    }

    if let Some(m) = level_re().find(rest) {
        let pre = &rest[..m.start()];
        let level = m.as_str();
        let post = &rest[m.end()..];
        if !pre.is_empty() {
            spans.push(Span::raw(pre.to_string()));
        }
        spans.push(Span::styled(level.to_string(), level_style(level)));
        if !post.is_empty() {
            spans.push(Span::raw(post.to_string()));
        }
        return Line::from(spans);
    }

    if spans.is_empty() {
        return Line::from(line.to_string());
    }
    spans.push(Span::raw(rest.to_string()));
    Line::from(spans)
}

fn level_style(level: &str) -> Style {
    match level {
        "ERROR" | "ERR" | "CRITICAL" | "FATAL" => {
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        }
        "WARN" | "WARNING" => Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
        "INFO" | "NOTICE" => Style::default().fg(Color::Blue),
        "DEBUG" | "TRACE" => Style::default().fg(Color::DarkGray),
        _ => Style::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_traceback_with_header() {
        let body = "Traceback (most recent call last):\n  File \"x.py\", line 1, in <module>\n    raise RuntimeError(\"oops\")\nRuntimeError: oops\n";
        assert!(looks_like_python_traceback(body));
    }

    #[test]
    fn detects_traceback_from_file_line_alone() {
        // Some loggers omit the canonical header.
        let body = "  File \"/app/main.py\", line 42, in main\n    do_thing()\n";
        assert!(looks_like_python_traceback(body));
    }

    #[test]
    fn does_not_mistake_plain_log_for_traceback() {
        let body = "2025-05-31 10:00:00 INFO Training started\n";
        assert!(!looks_like_python_traceback(body));
    }

    #[test]
    fn highlights_error_level() {
        let line = "2025-05-31 10:00:00 ERROR something broke";
        let l = highlight_line(line);
        // Three spans: timestamp, optional middle, ERROR; or merged.
        // We at least expect the ERROR span to be styled red.
        let has_error_span = l
            .spans
            .iter()
            .any(|s| s.content.as_ref() == "ERROR" && s.style.fg == Some(Color::Red));
        assert!(has_error_span, "ERROR token not red-styled");
    }

    #[test]
    fn highlights_file_line() {
        let line = r#"  File "/x.py", line 7, in foo"#;
        let l = highlight_line(line);
        // We expect at least the line number span styled yellow.
        let has_line_no = l
            .spans
            .iter()
            .any(|s| s.content.as_ref() == "7" && s.style.fg == Some(Color::Yellow));
        assert!(has_line_no, "line number not yellow-styled");
    }
}
