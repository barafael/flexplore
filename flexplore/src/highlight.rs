//! Lightweight syntax highlighting for the code preview panel.
//!
//! Produces an `egui::text::LayoutJob` with coloured spans for a given
//! source string and language hint.  The tokeniser is intentionally simple
//! (no full grammar — just keywords, strings, comments, numbers, and tags)
//! but gives enough colour contrast to make generated code scannable.

use bevy_egui::egui;
use egui::text::LayoutJob;
use egui::{Color32, FontId, TextFormat};

// ─── Language hint ───────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Rust,
    Html,
    Jsx,
    Swift,
    Dart,
}

pub fn lang_for_framework(idx: usize) -> Lang {
    match idx {
        0 => Lang::Rust,  // Bevy
        1 => Lang::Html,  // HTML/CSS
        2 => Lang::Html,  // Tailwind
        3 => Lang::Jsx,   // React
        4 => Lang::Swift, // SwiftUI
        5 => Lang::Dart,  // Flutter
        6 => Lang::Rust,  // Iced
        7 => Lang::Rust,  // egui
        8 => Lang::Jsx,   // React Native
        9 => Lang::Rust,  // Dioxus
        _ => Lang::Rust,
    }
}

// ─── Theme colours ───────────────────────────────────────────────────────────

struct Theme {
    keyword: Color32,
    string: Color32,
    comment: Color32,
    number: Color32,
    tag: Color32,
    property: Color32,
    r#type: Color32,
    punctuation: Color32,
    default: Color32,
}

fn dark_theme() -> Theme {
    Theme {
        keyword: Color32::from_rgb(0xC6, 0x78, 0xDD),  // purple
        string: Color32::from_rgb(0xA6, 0xE2, 0x2E),   // green
        comment: Color32::from_rgb(0x75, 0x71, 0x5E),  // gray
        number: Color32::from_rgb(0xFD, 0x97, 0x1F),   // orange
        tag: Color32::from_rgb(0x66, 0xD9, 0xEF),      // cyan
        property: Color32::from_rgb(0x66, 0xD9, 0xEF), // cyan
        r#type: Color32::from_rgb(0xE6, 0xDB, 0x74),   // yellow
        punctuation: Color32::from_rgb(0x88, 0x88, 0x88), // dim
        default: Color32::from_rgb(0xF8, 0xF8, 0xF2),  // off-white
    }
}

// ─── Keyword tables ──────────────────────────────────────────────────────────

const RUST_KW: &[&str] = &[
    "fn", "let", "mut", "pub", "use", "mod", "struct", "enum", "impl", "trait", "for", "in", "if",
    "else", "match", "return", "self", "Self", "super", "crate", "true", "false", "const",
    "static", "type", "where", "async", "await", "move", "ref", "as", "loop", "while", "break",
    "continue", "default", "unsafe", "dyn", "extern",
];

const SWIFT_KW: &[&str] = &[
    "func",
    "var",
    "let",
    "struct",
    "class",
    "enum",
    "import",
    "return",
    "if",
    "else",
    "for",
    "in",
    "while",
    "switch",
    "case",
    "self",
    "Self",
    "true",
    "false",
    "nil",
    "guard",
    "some",
    "private",
    "public",
    "static",
    "override",
    "protocol",
    "extension",
    "init",
    "deinit",
    "typealias",
];

const DART_KW: &[&str] = &[
    "void",
    "var",
    "final",
    "const",
    "class",
    "extends",
    "import",
    "return",
    "if",
    "else",
    "for",
    "in",
    "while",
    "switch",
    "case",
    "new",
    "this",
    "super",
    "true",
    "false",
    "null",
    "static",
    "abstract",
    "override",
    "Widget",
    "BuildContext",
    "Key",
];

const JS_KW: &[&str] = &[
    "function",
    "const",
    "let",
    "var",
    "return",
    "if",
    "else",
    "for",
    "in",
    "of",
    "while",
    "switch",
    "case",
    "new",
    "this",
    "true",
    "false",
    "null",
    "undefined",
    "export",
    "default",
    "import",
    "from",
    "class",
    "extends",
];

const CSS_PROPS: &[&str] = &[
    "display",
    "flex-direction",
    "flex-wrap",
    "justify-content",
    "align-items",
    "align-content",
    "align-self",
    "flex-grow",
    "flex-shrink",
    "flex-basis",
    "grid-template-columns",
    "grid-template-rows",
    "grid-auto-flow",
    "grid-auto-columns",
    "grid-auto-rows",
    "grid-column",
    "grid-row",
    "width",
    "height",
    "min-width",
    "min-height",
    "max-width",
    "max-height",
    "padding",
    "margin",
    "border-width",
    "border-radius",
    "border-style",
    "row-gap",
    "column-gap",
    "gap",
    "order",
    "background",
    "color",
    "font-size",
    "box-sizing",
    "visibility",
    "overflow",
];

// ─── Public API ──────────────────────────────────────────────────────────────

pub fn highlight(code: &str, lang: Lang, font: FontId) -> LayoutJob {
    let theme = dark_theme();
    let mut job = LayoutJob::default();
    job.wrap.max_width = f32::INFINITY;

    match lang {
        Lang::Html => highlight_html(code, &theme, &font, &mut job),
        Lang::Rust => highlight_rust(code, &theme, &font, &mut job),
        Lang::Jsx => highlight_jsx(code, &theme, &font, &mut job),
        Lang::Swift => highlight_c_like(code, SWIFT_KW, &theme, &font, &mut job),
        Lang::Dart => highlight_c_like(code, DART_KW, &theme, &font, &mut job),
    }

    job
}

// ─── Append helpers ──────────────────────────────────────────────────────────

fn push(job: &mut LayoutJob, text: &str, color: Color32, font: &FontId) {
    if text.is_empty() {
        return;
    }
    job.append(
        text,
        0.0,
        TextFormat {
            font_id: font.clone(),
            color,
            ..Default::default()
        },
    );
}

// ─── HTML / CSS highlighter ──────────────────────────────────────────────────

fn highlight_html(src: &str, theme: &Theme, font: &FontId, job: &mut LayoutJob) {
    let mut chars = src.char_indices().peekable();

    while let Some(&(i, ch)) = chars.peek() {
        if ch == '<' {
            // HTML tag
            let start = i;
            let mut end = i;
            while let Some(&(j, c)) = chars.peek() {
                end = j + c.len_utf8();
                chars.next();
                if c == '>' {
                    break;
                }
            }
            push(job, &src[start..end], theme.tag, font);
        } else if src[i..].starts_with("/*") {
            // Block comment
            let start = i;
            chars.next();
            chars.next();
            let mut end = i + 2;
            while let Some(&(j, c)) = chars.peek() {
                end = j + c.len_utf8();
                chars.next();
                if c == '/' && j > 0 && src.as_bytes().get(j - 1) == Some(&b'*') {
                    break;
                }
            }
            push(job, &src[start..end], theme.comment, font);
        } else if src[i..].starts_with("//") {
            // Line comment
            let start = i;
            let end = src[i..].find('\n').map_or(src.len(), |n| i + n);
            push(job, &src[start..end], theme.comment, font);
            // Advance past
            while let Some(&(j, _)) = chars.peek() {
                if j >= end {
                    break;
                }
                chars.next();
            }
        } else if ch == '"' || ch == '\'' {
            // String literal
            let quote = ch;
            let start = i;
            chars.next();
            while let Some(&(_, c)) = chars.peek() {
                chars.next();
                if c == quote {
                    break;
                }
                if c == '\\' {
                    chars.next(); // skip escaped char
                }
            }
            let end = chars.peek().map_or(src.len(), |&(j, _)| j);
            push(job, &src[start..end], theme.string, font);
        } else if ch.is_ascii_digit()
            || (ch == '-'
                && src[i..].len() > 1
                && src
                    .as_bytes()
                    .get(i + 1)
                    .is_some_and(|b| b.is_ascii_digit()))
        {
            // Number
            let start = i;
            if ch == '-' {
                chars.next();
            }
            while let Some(&(_, c)) = chars.peek() {
                if c.is_ascii_digit() || c == '.' {
                    chars.next();
                } else {
                    break;
                }
            }
            // Consume unit suffix (px, %, fr, vw, vh, em, rem)
            let pos = chars.peek().map_or(src.len(), |&(j, _)| j);
            let rest = &src[pos..];
            let units = ["px", "fr", "vw", "vh", "%"];
            let mut unit_len = 0;
            for u in units {
                if rest.starts_with(u) {
                    unit_len = u.len();
                    break;
                }
            }
            for _ in 0..unit_len {
                chars.next();
            }
            let end = chars.peek().map_or(src.len(), |&(j, _)| j);
            push(job, &src[start..end], theme.number, font);
        } else if ch.is_ascii_alphabetic() || ch == '_' || ch == '-' {
            // Word (could be CSS property, tag name, etc.)
            let start = i;
            while let Some(&(_, c)) = chars.peek() {
                if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                    chars.next();
                } else {
                    break;
                }
            }
            let end = chars.peek().map_or(src.len(), |&(j, _)| j);
            let word = &src[start..end];

            // Check if followed by `:` (CSS property)
            let is_css_prop =
                chars.peek().is_some_and(|&(_, c)| c == ':') && CSS_PROPS.contains(&word);

            if is_css_prop {
                push(job, word, theme.property, font);
            } else {
                push(job, word, theme.default, font);
            }
        } else {
            // Punctuation / whitespace
            chars.next();
            let end = i + ch.len_utf8();
            let color = if ch.is_ascii_punctuation() && ch != '_' {
                theme.punctuation
            } else {
                theme.default
            };
            push(job, &src[i..end], color, font);
        }
    }
}

// ─── Rust highlighter ────────────────────────────────────────────────────────

fn highlight_rust(src: &str, theme: &Theme, font: &FontId, job: &mut LayoutJob) {
    highlight_c_like(src, RUST_KW, theme, font, job);
}

// ─── JSX highlighter ─────────────────────────────────────────────────────────

fn highlight_jsx(src: &str, theme: &Theme, font: &FontId, job: &mut LayoutJob) {
    let mut chars = src.char_indices().peekable();

    while let Some(&(i, ch)) = chars.peek() {
        if ch == '<' && src[i..].len() > 1 {
            // JSX tag
            let next = src.as_bytes().get(i + 1).copied().unwrap_or(0);
            if next.is_ascii_alphabetic() || next == b'/' {
                let start = i;
                let mut end = i;
                while let Some(&(j, c)) = chars.peek() {
                    end = j + c.len_utf8();
                    chars.next();
                    if c == '>' {
                        break;
                    }
                }
                push(job, &src[start..end], theme.tag, font);
                continue;
            }
        }

        if src[i..].starts_with("//") {
            let start = i;
            let end = src[i..].find('\n').map_or(src.len(), |n| i + n);
            push(job, &src[start..end], theme.comment, font);
            while let Some(&(j, _)) = chars.peek() {
                if j >= end {
                    break;
                }
                chars.next();
            }
        } else if ch == '"' || ch == '\'' || ch == '`' {
            let quote = ch;
            let start = i;
            chars.next();
            while let Some(&(_, c)) = chars.peek() {
                chars.next();
                if c == quote {
                    break;
                }
                if c == '\\' {
                    chars.next();
                }
            }
            let end = chars.peek().map_or(src.len(), |&(j, _)| j);
            push(job, &src[start..end], theme.string, font);
        } else if ch.is_ascii_digit() {
            let start = i;
            while let Some(&(_, c)) = chars.peek() {
                if c.is_ascii_digit() || c == '.' {
                    chars.next();
                } else {
                    break;
                }
            }
            let end = chars.peek().map_or(src.len(), |&(j, _)| j);
            push(job, &src[start..end], theme.number, font);
        } else if ch.is_ascii_alphabetic() || ch == '_' || ch == '$' {
            let start = i;
            while let Some(&(_, c)) = chars.peek() {
                if c.is_ascii_alphanumeric() || c == '_' || c == '$' {
                    chars.next();
                } else {
                    break;
                }
            }
            let end = chars.peek().map_or(src.len(), |&(j, _)| j);
            let word = &src[start..end];
            if JS_KW.contains(&word) {
                push(job, word, theme.keyword, font);
            } else if word.starts_with(|c: char| c.is_ascii_uppercase()) {
                push(job, word, theme.r#type, font);
            } else {
                push(job, word, theme.default, font);
            }
        } else {
            chars.next();
            let end = i + ch.len_utf8();
            let color = if ch.is_ascii_punctuation() {
                theme.punctuation
            } else {
                theme.default
            };
            push(job, &src[i..end], color, font);
        }
    }
}

// ─── C-like (Rust / Swift / Dart) highlighter ────────────────────────────────

fn highlight_c_like(
    src: &str,
    keywords: &[&str],
    theme: &Theme,
    font: &FontId,
    job: &mut LayoutJob,
) {
    let mut chars = src.char_indices().peekable();

    while let Some(&(i, ch)) = chars.peek() {
        if src[i..].starts_with("//") {
            let start = i;
            let end = src[i..].find('\n').map_or(src.len(), |n| i + n);
            push(job, &src[start..end], theme.comment, font);
            while let Some(&(j, _)) = chars.peek() {
                if j >= end {
                    break;
                }
                chars.next();
            }
        } else if ch == '"' {
            let start = i;
            chars.next();
            while let Some(&(_, c)) = chars.peek() {
                chars.next();
                if c == '"' {
                    break;
                }
                if c == '\\' {
                    chars.next();
                }
            }
            let end = chars.peek().map_or(src.len(), |&(j, _)| j);
            push(job, &src[start..end], theme.string, font);
        } else if ch.is_ascii_digit() {
            let start = i;
            while let Some(&(_, c)) = chars.peek() {
                if c.is_ascii_digit() || c == '.' || c == '_' {
                    chars.next();
                } else {
                    break;
                }
            }
            let end = chars.peek().map_or(src.len(), |&(j, _)| j);
            push(job, &src[start..end], theme.number, font);
        } else if ch.is_ascii_alphabetic() || ch == '_' {
            let start = i;
            while let Some(&(_, c)) = chars.peek() {
                if c.is_ascii_alphanumeric() || c == '_' {
                    chars.next();
                } else {
                    break;
                }
            }
            let end = chars.peek().map_or(src.len(), |&(j, _)| j);
            let word = &src[start..end];
            if keywords.contains(&word) {
                push(job, word, theme.keyword, font);
            } else if word.starts_with(|c: char| c.is_ascii_uppercase()) {
                push(job, word, theme.r#type, font);
            } else {
                push(job, word, theme.default, font);
            }
        } else {
            chars.next();
            let end = i + ch.len_utf8();
            let color = if ch.is_ascii_punctuation() && ch != '_' {
                theme.punctuation
            } else {
                theme.default
            };
            push(job, &src[i..end], color, font);
        }
    }
}
