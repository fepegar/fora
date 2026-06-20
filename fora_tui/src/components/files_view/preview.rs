//! Preview pane: classify the bytes, then render them.
//!
//! Classification (cheap, runs on the UI thread when an action arrives):
//!
//! - **Binary** — scan the first 8 KB for a `NUL`; treat as binary.
//! - **Image** — extension matches one of PNG/JPEG/GIF/WebP.
//! - **TooLarge** — over the plain-text cap.
//! - **AnsiText** — log files, or any UTF-8 body containing ANSI escape
//!   sequences (`ESC [`).
//! - **HighlightedText** — within the highlight caps; runs syntect.
//! - **PlainText** — fallback; UTF-8 decoded with `from_utf8_lossy`.
//!
//! Rendering of an already-classified `PreviewKind` is a slice of
//! pre-built `Line<'static>`s into a `Paragraph`.

use bytes::Bytes;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{
    Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
};
use ratatui::Frame;

use super::highlight::{ansi_to_lines, plain_lines, HIGHLIGHT_MAX_BYTES, HIGHLIGHT_MAX_LINES};
use super::state::{FilesState, PaneFocus, PreviewState};
use crate::theme::Theme;

const BINARY_SCAN_BYTES: usize = 8 * 1024;
const PLAIN_TEXT_MAX_BYTES: usize = 10 * 1024 * 1024;
const IMAGE_EXTS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp"];

/// Decoded form of a file preview. Bytes are kept separately on the
/// [`super::state::CachedPreview`]; this enum carries any UI-ready
/// representation.
#[derive(Debug, Clone)]
pub enum PreviewKind {
    PlainText {
        lines: Vec<Line<'static>>,
    },
    AnsiText {
        lines: Vec<Line<'static>>,
    },
    Highlighted {
        lines: Vec<Line<'static>>,
    },
    Binary {
        mime: String,
        size: u64,
    },
    /// Image bytes decoded once into a `DynamicImage` so the render path
    /// only has to map it to the terminal's graphics protocol.
    Image {
        ext: String,
        /// Decoded image, kept around so we can hand it to ratatui-image
        /// at render time. `None` if decoding failed; we fall back to the
        /// Binary preview in that case.
        decoded: Option<std::sync::Arc<image::DynamicImage>>,
    },
    TooLarge {
        size: u64,
    },
}

/// Classify a freshly-arrived preview body using the default theme.
pub fn classify(path: &str, bytes: &Bytes) -> PreviewKind {
    classify_with_theme(path, bytes, "base16-ocean.dark")
}

/// Like [`classify`] but lets the caller pick the syntect theme name
/// used for highlighting.
pub fn classify_with_theme(path: &str, bytes: &Bytes, theme_name: &str) -> PreviewKind {
    let size = bytes.len() as u64;
    let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();

    if IMAGE_EXTS.contains(&ext.as_str()) {
        let decoded = decode_image_limited(bytes).map(std::sync::Arc::new);
        return PreviewKind::Image { ext, decoded };
    }

    if (bytes.len() as u64) > PLAIN_TEXT_MAX_BYTES as u64 {
        return PreviewKind::TooLarge { size };
    }

    if looks_binary(bytes) {
        let mime = mime_guess::from_path(path)
            .first_raw()
            .unwrap_or("application/octet-stream")
            .to_string();
        return PreviewKind::Binary { mime, size };
    }

    let text = String::from_utf8_lossy(bytes);
    let has_ansi = contains_ansi_escape(&text);
    let is_log = is_log_path(path);

    // Logs with embedded ANSI escapes (tqdm, rich, …) keep using the
    // ANSI pipeline so we don't trample the colours the framework already
    // emitted.
    if has_ansi {
        return PreviewKind::AnsiText {
            lines: ansi_to_lines(&text),
        };
    }

    // Logs without ANSI escapes get the manual line-by-line log
    // colouring (Python traceback, log levels, timestamps).
    if is_log {
        return PreviewKind::AnsiText {
            lines: super::log_highlight::highlight_log_text(&text),
        };
    }

    let line_count = text.lines().count();
    if bytes.len() <= HIGHLIGHT_MAX_BYTES && line_count <= HIGHLIGHT_MAX_LINES {
        return PreviewKind::Highlighted {
            lines: super::highlight::highlight_text_themed(path, &text, theme_name),
        };
    }

    PreviewKind::PlainText {
        lines: plain_lines(&text),
    }
}

fn looks_binary(bytes: &Bytes) -> bool {
    let scan_end = BINARY_SCAN_BYTES.min(bytes.len());
    bytes[..scan_end].contains(&0u8)
}

/// Decode an image with explicit resource limits so a small but highly
/// compressed file can't expand into a huge pixel buffer (a decompression
/// bomb) and OOM the TUI.
fn decode_image_limited(bytes: &Bytes) -> Option<image::DynamicImage> {
    use std::io::Cursor;

    let mut limits = image::Limits::default();
    limits.max_image_width = Some(16_384);
    limits.max_image_height = Some(16_384);
    limits.max_alloc = Some(256 * 1024 * 1024);

    let mut reader = image::ImageReader::new(Cursor::new(bytes.as_ref()))
        .with_guessed_format()
        .ok()?;
    reader.limits(limits);
    reader.decode().ok()
}

fn contains_ansi_escape(text: &str) -> bool {
    text.as_bytes().windows(2).any(|w| w == [0x1b, b'['])
}

pub(super) fn is_log_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    let leaf = lower.rsplit('/').next().unwrap_or(&lower);
    lower.ends_with(".log")
        || leaf.starts_with("std_log")
        || lower.starts_with("user_logs/")
        || lower.starts_with("system_logs/")
        || lower.contains("/user_logs/")
        || lower.contains("/system_logs/")
}

/// Render the preview pane into `area`.
pub fn render_preview(frame: &mut Frame, area: Rect, state: &mut FilesState) {
    let focus_border = if matches!(state.focus, PaneFocus::Preview) {
        Theme::BORDER_ACTIVE
    } else {
        Theme::BORDER
    };
    let title = match state.previewed_path() {
        Some(p) => format!(" {} ", super::state::leaf_name(p)),
        None => " Preview ".to_string(),
    };
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(focus_border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width < 2 || inner.height < 1 {
        return;
    }

    let Some(path) = state.previewed_path().cloned() else {
        let msg = Paragraph::new(" Select a file in the tree to preview ")
            .style(Style::default().fg(Theme::DIM));
        frame.render_widget(msg, inner);
        return;
    };

    let preview = state.previews.peek(&path).cloned();
    match preview {
        None => {
            let msg = Paragraph::new(" Loading preview… ").style(Style::default().fg(Theme::DIM));
            frame.render_widget(msg, inner);
        }
        Some(PreviewState::Loading) => {
            let msg = Paragraph::new(" Loading preview… ").style(Style::default().fg(Theme::DIM));
            frame.render_widget(msg, inner);
        }
        Some(PreviewState::Error(err)) => {
            let msg = Paragraph::new(format!(" Error: {}", err))
                .style(Style::default().fg(Theme::ERROR))
                .wrap(Wrap { trim: false });
            frame.render_widget(msg, inner);
        }
        Some(PreviewState::Loaded(p)) => {
            render_loaded(frame, inner, state, &p);
        }
    }
}

fn render_loaded(
    frame: &mut Frame,
    area: Rect,
    state: &mut FilesState,
    p: &super::state::CachedPreview,
) {
    match &p.kind {
        PreviewKind::PlainText { lines }
        | PreviewKind::AnsiText { lines }
        | PreviewKind::Highlighted { lines } => {
            // Build the paragraph once so we can ask ratatui itself how
            // many visual rows it will produce. Doing the wrap-counting
            // by hand (ceil(line.width()/area.width)) is unreliable
            // because `WordWrapper` is word-aware: a single source line
            // can collapse to fewer or expand to more visual rows than
            // a pure ceil would predict, breaking "scroll to bottom".
            //
            // `Paragraph::line_count` is feature-gated behind
            // `unstable-rendered-line-info`, enabled in the workspace
            // Cargo.toml. It uses the same `WordWrapper` as the render
            // path, so the count is exact for any character width.
            let paragraph = Paragraph::new(lines.clone()).wrap(Wrap { trim: false });
            let visual_rows = paragraph.line_count(area.width);
            let max_scroll: u16 = visual_rows
                .saturating_sub(area.height as usize)
                .min(u16::MAX as usize) as u16;
            let scroll = state.preview_scroll.min(max_scroll);
            state.preview_scroll = scroll;
            // Re-arm auto-follow if the user scrolled (or paged) back to
            // the bottom of a log file.
            if scroll == max_scroll
                && state
                    .previewed_path()
                    .map(|p| super::preview::is_log_path(p))
                    .unwrap_or(false)
            {
                state.auto_tail_to_bottom = true;
            }
            frame.render_widget(paragraph.scroll((scroll, 0)), area);
            if visual_rows > area.height as usize {
                let max = max_scroll as usize;
                let mut sb_state = ScrollbarState::new(max).position(scroll as usize);
                let sb = Scrollbar::new(ScrollbarOrientation::VerticalRight);
                frame.render_stateful_widget(sb, area, &mut sb_state);
            }
        }
        PreviewKind::Binary { mime, size } => {
            let msg = Paragraph::new(format!(
                " Binary file ({} bytes)\n MIME: {}\n\n Press s to save to disk.",
                size, mime,
            ))
            .style(
                Style::default()
                    .fg(Theme::DIM)
                    .add_modifier(Modifier::ITALIC),
            )
            .wrap(Wrap { trim: false });
            frame.render_widget(msg, area);
        }
        PreviewKind::Image { ext, decoded } => match decoded {
            Some(img) => render_image(frame, area, state, img),
            None => {
                let msg = Paragraph::new(format!(
                    " Image ({} bytes, .{}) — failed to decode.\n\n Press s to save to disk.",
                    p.bytes.len(),
                    ext,
                ))
                .style(Style::default().fg(Theme::ERROR))
                .wrap(Wrap { trim: false });
                frame.render_widget(msg, area);
            }
        },
        PreviewKind::TooLarge { size } => {
            let msg = Paragraph::new(format!(
                " File is {} bytes ({} MiB) — too large for in-pane preview.\n\n Press s to save to disk.",
                size,
                size / (1024 * 1024),
            ))
            .style(Style::default().fg(Theme::WARNING))
            .wrap(Wrap { trim: false });
            frame.render_widget(msg, area);
        }
    }
}

fn render_image(
    frame: &mut Frame,
    area: Rect,
    state: &mut FilesState,
    img: &std::sync::Arc<image::DynamicImage>,
) {
    if state.picker.is_none() {
        state.picker = Some(
            ratatui_image::picker::Picker::from_query_stdio()
                .unwrap_or_else(|_| ratatui_image::picker::Picker::halfblocks()),
        );
    }
    let picker = state.picker.as_mut().unwrap();
    let font = picker.font_size();
    let target = ratatui::layout::Size::new(area.width, area.height);
    // Constrain image size to the available cells; ratatui-image internally
    // rescales pixels-per-cell using font_size.
    let _ = font; // currently unused; protocol does the math internally.
    match picker.new_protocol((**img).clone(), target, ratatui_image::Resize::Fit(None)) {
        Ok(protocol) => {
            let widget = ratatui_image::Image::new(&protocol);
            frame.render_widget(widget, area);
        }
        Err(e) => {
            let msg = Paragraph::new(format!(" Image render failed: {}", e))
                .style(Style::default().fg(Theme::ERROR))
                .wrap(Wrap { trim: false });
            frame.render_widget(msg, area);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detects_image_by_extension() {
        let kind = classify("plots/loss.png", &Bytes::from_static(b"\x89PNG\r\n\x1a\n"));
        assert!(matches!(kind, PreviewKind::Image { .. }));
    }

    #[test]
    fn detects_binary_via_null_byte() {
        let bytes = Bytes::from(vec![1u8, 2, 0, 3, 4]);
        let kind = classify("blob.bin", &bytes);
        assert!(matches!(kind, PreviewKind::Binary { .. }));
    }

    #[test]
    fn ansi_in_user_logs_uses_ansi_pipeline() {
        let body = b"user_logs/std_log.txt content with \x1b[31mred\x1b[0m";
        let kind = classify("user_logs/std_log.txt", &Bytes::copy_from_slice(body));
        assert!(matches!(kind, PreviewKind::AnsiText { .. }));
    }

    #[test]
    fn std_log_prefixed_filename_is_detected_as_log() {
        assert!(is_log_path("user_logs/std_log_process_0.txt"));
        assert!(is_log_path("std_log_0_0.txt"));
        assert!(is_log_path("std_log.txt"));
        assert!(!is_log_path("notes.txt"));
    }

    #[test]
    fn plain_log_without_ansi_still_goes_through_text_pipeline() {
        // Lives under user_logs/ — routed via the log highlighter; the
        // outward-facing kind is still `AnsiText` (a Vec<Line>) so the
        // renderer can treat it uniformly.
        let body = b"2025-01-01 INFO Training started\n";
        let kind = classify(
            "user_logs/std_log_process_0.txt",
            &Bytes::copy_from_slice(body),
        );
        assert!(matches!(kind, PreviewKind::AnsiText { .. }));
    }

    #[test]
    fn small_python_file_is_highlighted() {
        let body = b"def foo(): return 42\n";
        let kind = classify("train.py", &Bytes::copy_from_slice(body));
        assert!(matches!(kind, PreviewKind::Highlighted { .. }));
    }

    #[test]
    fn oversize_file_is_too_large() {
        let body = vec![b'a'; PLAIN_TEXT_MAX_BYTES + 1];
        let kind = classify("dump.txt", &Bytes::from(body));
        assert!(matches!(kind, PreviewKind::TooLarge { .. }));
    }

    #[test]
    fn utf8_bom_is_not_classified_binary() {
        let mut body = vec![0xEF, 0xBB, 0xBF];
        body.extend_from_slice(b"hello");
        let kind = classify("readme.txt", &Bytes::from(body));
        assert!(!matches!(kind, PreviewKind::Binary { .. }));
    }
}
