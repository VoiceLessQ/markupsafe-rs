//! A Rust port of Python's [`markupsafe`](https://github.com/pallets/markupsafe) — HTML
//! escaping with a string type that tracks whether its contents are already safe.
//!
//! Port target: `../Reference/markupsafe/src/markupsafe/__init__.py` (and `_native.py`).
//!
//! The security idea: text that has been escaped (or explicitly marked safe) is a distinct
//! type, [`Markup`]. In Python `Markup` is a `str` subclass that enforces this by convention;
//! here it is a newtype, so "this text is safe to embed in HTML" is a compile-time guarantee.

use std::fmt;
use std::ops::{Add, Deref};

/// The HTML character escaping, a port of `_native.py`'s `_escape_inner`.
///
/// Python applies five ordered `str.replace` passes (with `&` first, so the `&` it inserts
/// for the other entities is not re-escaped). A single pass over the characters produces the
/// identical result — and, since each input character is handled exactly once, the ordering
/// subtlety disappears entirely. This is the one-pass equivalent of the C speed-up module.
fn escape_inner(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '>' => out.push_str("&gt;"),
            '<' => out.push_str("&lt;"),
            '\'' => out.push_str("&#39;"),
            '"' => out.push_str("&#34;"),
            _ => out.push(ch),
        }
    }
    out
}

/// A type that can render itself as already-safe HTML. Port of the `__html__` protocol.
pub trait HasHtml {
    /// Return an HTML-safe representation of `self`.
    fn to_html(&self) -> Markup;
}

/// Escape the `&`, `<`, `>`, `'`, and `"` characters in `s` so it is safe to embed in HTML.
/// Port of the module-level `escape`.
pub fn escape(s: &str) -> Markup {
    Markup(escape_inner(s))
}

/// Escape a value that knows how to render itself as HTML (`__html__`): its output is taken
/// as already safe and wrapped without further escaping. Port of `escape`'s `__html__` branch.
pub fn escape_html<T: HasHtml + ?Sized>(value: &T) -> Markup {
    value.to_html()
}

/// Like [`escape`] but treats `None` as the empty string rather than the text `"None"`.
/// Port of `escape_silent`.
pub fn escape_silent(s: Option<&str>) -> Markup {
    match s {
        None => Markup::new(""),
        Some(s) => escape(s),
    }
}

/// A string that is safe to insert into HTML — either because it was escaped or because it
/// was explicitly marked safe. Port of `markupsafe.Markup`.
///
/// Read-only `str` methods are available through [`Deref`]; combining a `Markup` with a plain
/// `&str` escapes the `&str`, while combining two `Markup`s does not (both are already safe).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Markup(String);

impl Markup {
    /// Mark text as safe **without escaping it**. The caller asserts it is already safe HTML
    /// (e.g. a trusted literal). Equivalent to `Markup("...")` in Python.
    pub fn new(s: impl Into<String>) -> Markup {
        Markup(s.into())
    }

    /// Escape `s` and wrap the result. Equivalent to `Markup.escape(...)`.
    pub fn escape(s: &str) -> Markup {
        escape(s)
    }

    /// The safe contents as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume the `Markup`, returning the inner `String`.
    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for Markup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Deref for Markup {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl HasHtml for Markup {
    /// A `Markup` is already safe, so it renders as itself.
    fn to_html(&self) -> Markup {
        self.clone()
    }
}

// Combining safe + unsafe escapes the unsafe side; safe + safe does not. This is the
// security-relevant behaviour of `Markup.__add__`.
impl Add<&str> for Markup {
    type Output = Markup;
    fn add(self, rhs: &str) -> Markup {
        Markup(self.0 + &escape_inner(rhs))
    }
}

impl Add<Markup> for Markup {
    type Output = Markup;
    fn add(mut self, rhs: Markup) -> Markup {
        self.0.push_str(&rhs.0);
        self
    }
}

impl Add<&Markup> for Markup {
    type Output = Markup;
    fn add(mut self, rhs: &Markup) -> Markup {
        self.0.push_str(&rhs.0);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_the_five_characters() {
        assert_eq!(escape("<>&'\"").as_str(), "&lt;&gt;&amp;&#39;&#34;");
        assert_eq!(
            escape("Hello, <em>World</em>!").as_str(),
            "Hello, &lt;em&gt;World&lt;/em&gt;!"
        );
    }

    #[test]
    fn ampersand_is_not_double_escaped() {
        // Each input character is escaped exactly once.
        assert_eq!(escape("a & b").as_str(), "a &amp; b");
        // Escaping is not idempotent: re-escaping doubles the entity (matches Python).
        assert_eq!(escape("&amp;").as_str(), "&amp;amp;");
    }

    #[test]
    fn plain_text_passes_through() {
        assert_eq!(escape("safe text 123").as_str(), "safe text 123");
    }

    #[test]
    fn new_marks_safe_without_escaping() {
        assert_eq!(Markup::new("<b>bold</b>").as_str(), "<b>bold</b>");
        assert_eq!(Markup::new("<b>bold</b>").to_string(), "<b>bold</b>");
    }

    #[test]
    fn concatenation_escapes_only_the_unsafe_side() {
        // safe + &str escapes the &str
        let combined = Markup::new("<em>Hello</em> ") + "<foo>";
        assert_eq!(combined.as_str(), "<em>Hello</em> &lt;foo&gt;");
        // safe + safe does not escape
        let both = Markup::new("<a>") + Markup::new("<b>");
        assert_eq!(both.as_str(), "<a><b>");
    }

    #[test]
    fn escape_silent_handles_none() {
        assert_eq!(escape_silent(None).as_str(), "");
        assert_eq!(escape_silent(Some("<")).as_str(), "&lt;");
    }

    #[test]
    fn has_html_protocol() {
        let m = Markup::new("<a href=\"/foo\">foo</a>");
        // A type that is already HTML renders as itself, unescaped.
        assert_eq!(escape_html(&m).as_str(), "<a href=\"/foo\">foo</a>");
    }

    #[test]
    fn deref_exposes_read_only_str_methods() {
        let m = escape("<x>");
        assert_eq!(m.len(), "&lt;x&gt;".len());
        assert!(m.contains("&lt;"));
    }
}
