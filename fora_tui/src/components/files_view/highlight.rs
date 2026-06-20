//! syntect-based syntax highlighting adapter.
//!
//! Returns ratatui `Line<'static>` rows already styled so the render path
//! is a cheap clone of a pre-built `Vec<Line>`. We only highlight under
//! the caps in [`super::preview`] — anything beyond falls back to plain
//! text in the classifier.
//!
//! The first call into [`HIGHLIGHTER`] loads syntect's default
//! `SyntaxSet` and `ThemeSet` (~3 MiB of zero-copy embedded data); after
//! that highlighting is a per-line linear scan, fast enough to do on the
//! UI thread for files within our caps.

use std::sync::OnceLock;

use ansi_to_tui::IntoText;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, Style as SynStyle, Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;

const DEFAULT_THEME: &str = "base16-ocean.dark";

struct Highlighter {
    syntaxes: SyntaxSet,
    themes: ThemeSet,
}

static HIGHLIGHTER: OnceLock<Highlighter> = OnceLock::new();

fn highlighter() -> &'static Highlighter {
    HIGHLIGHTER.get_or_init(|| Highlighter {
        syntaxes: SyntaxSet::load_defaults_newlines(),
        themes: ThemeSet::load_defaults(),
    })
}

/// Pick a syntect syntax based on the file path's extension or first
/// line, falling back to plain text.
pub fn pick_syntax(path: &str, first_line: &str) -> &'static SyntaxReference {
    let h = highlighter();
    if let Some(ext) = path.rsplit('.').next() {
        if let Some(syntax) = h.syntaxes.find_syntax_by_extension(ext) {
            return syntax;
        }
    }
    if let Some(name) = path.rsplit('/').next() {
        if let Some(syntax) = h.syntaxes.find_syntax_by_name(name) {
            return syntax;
        }
    }
    if let Some(syntax) = h.syntaxes.find_syntax_by_first_line(first_line) {
        return syntax;
    }
    h.syntaxes.find_syntax_plain_text()
}

fn theme_named(name: &str) -> &'static Theme {
    let h = highlighter();
    h.themes
        .themes
        .get(name)
        .or_else(|| h.themes.themes.get(DEFAULT_THEME))
        .or_else(|| h.themes.themes.values().next())
        .expect("syntect bundles at least one theme")
}

#[allow(dead_code)]
fn theme() -> &'static Theme {
    theme_named(DEFAULT_THEME)
}

/// Highlight a complete file body, returning a `Vec<Line>` ready for
/// `Paragraph::new`. Lines are owned `Line<'static>` so the result can
/// be cached and re-used without lifetime headaches.
pub fn highlight_text(path: &str, body: &str) -> Vec<Line<'static>> {
    highlight_text_themed(path, body, DEFAULT_THEME)
}

/// Variant of [`highlight_text`] that allows the caller to pick the
/// syntect theme by name. Unknown theme names silently fall back to the
/// default.
pub fn highlight_text_themed(path: &str, body: &str, theme_name: &str) -> Vec<Line<'static>> {
    let h = highlighter();
    let first_line = body.lines().next().unwrap_or("");
    let syntax = pick_syntax(path, first_line);
    let theme = theme_named(theme_name);
    let mut highlighter = HighlightLines::new(syntax, theme);

    let mut out: Vec<Line<'static>> = Vec::with_capacity(body.lines().count());
    for line in LinesWithEndings::from(body) {
        match highlighter.highlight_line(line, &h.syntaxes) {
            Ok(ranges) => out.push(to_ratatui_line(&ranges)),
            Err(_) => out.push(Line::from(strip_trailing_newline(line).to_string())),
        }
    }
    out
}

/// Parse an ANSI-coloured text body into a ratatui `Text`. Falls back to
/// plain wrapping on parse failure.
pub fn ansi_to_lines(body: &str) -> Vec<Line<'static>> {
    match body.as_bytes().into_text() {
        Ok(text) => text.lines.into_iter().map(force_static_line).collect(),
        Err(_) => body.lines().map(|s| Line::from(s.to_string())).collect(),
    }
}

/// Render a plain (unstyled) text body to `Vec<Line>`.
pub fn plain_lines(body: &str) -> Vec<Line<'static>> {
    body.lines().map(|s| Line::from(s.to_string())).collect()
}

fn to_ratatui_line(ranges: &[(SynStyle, &str)]) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::with_capacity(ranges.len());
    for (style, text) in ranges {
        let stripped = strip_trailing_newline(text);
        if stripped.is_empty() {
            continue;
        }
        spans.push(Span::styled(stripped.to_string(), syn_to_ratatui(style)));
    }
    Line::from(spans)
}

fn syn_to_ratatui(style: &SynStyle) -> Style {
    let mut out = Style::default().fg(Color::Rgb(
        style.foreground.r,
        style.foreground.g,
        style.foreground.b,
    ));
    if style.font_style.contains(FontStyle::BOLD) {
        out = out.add_modifier(Modifier::BOLD);
    }
    if style.font_style.contains(FontStyle::ITALIC) {
        out = out.add_modifier(Modifier::ITALIC);
    }
    if style.font_style.contains(FontStyle::UNDERLINE) {
        out = out.add_modifier(Modifier::UNDERLINED);
    }
    out
}

fn strip_trailing_newline(s: &str) -> &str {
    s.strip_suffix('\n')
        .map(|s| s.strip_suffix('\r').unwrap_or(s))
        .unwrap_or(s)
}

/// Convert a borrowed `Line` into a fully owned `Line<'static>`. ratatui
/// returns Cow-backed spans from ansi-to-tui; we materialise them so the
/// result can sit in a cache without lifetime issues.
fn force_static_line(line: Line<'_>) -> Line<'static> {
    let spans: Vec<Span<'static>> = line
        .spans
        .into_iter()
        .map(|s| Span {
            content: s.content.into_owned().into(),
            style: s.style,
        })
        .collect();
    Line::from(spans).style(line.style)
}

/// Total work done above this size makes syntect noticeably laggy in
/// release builds. We let callers gate on this.
pub const HIGHLIGHT_MAX_BYTES: usize = 1024 * 1024;

/// Even short files with thousands of lines tank highlighting throughput.
pub const HIGHLIGHT_MAX_LINES: usize = 5_000;

// Re-exported only to silence dead-code lints in early integration; the
// raw `Text` variant is currently unused outside this crate.
#[allow(dead_code)]
fn _force_used(_: Text<'static>) {}
