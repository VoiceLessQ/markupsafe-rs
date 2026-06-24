# markupsafe-rs

A Rust port of Python's [`markupsafe`](https://github.com/pallets/markupsafe): HTML escaping
with a string type that tracks whether its contents are already safe. The escaping matches the
reference implementation byte-for-byte (verified by differential testing).

In Python, `Markup` is a `str` subclass that enforces "this text is safe" by convention. Here
it's a **newtype**, so safety is a compile-time guarantee: combining a `Markup` with raw text
forces the raw text to be escaped, making it impossible to accidentally treat unescaped input as safe.

## Features

- `escape`: replace `&`, `<`, `>`, `'`, `"` with HTML-safe entities, returning a `Markup`.
- `Markup`: a string known to be safe (escaped or explicitly marked). Combining it with a
  plain `&str` escapes the `&str`; combining two `Markup`s does not.
- `escape_silent` (treats `None` as empty), `escape_html` (for types implementing `HasHtml`),
  and the `HasHtml` trait (the `__html__` protocol).

## Installation

```sh
cargo add markupsafe-rs
```

```toml
[dependencies]
markupsafe-rs = "0.1"
```

Requires a Rust toolchain with 2024-edition support (Rust 1.85 or newer).

## Usage

```rust
use markupsafe_rs::{escape, Markup};

// Escape untrusted text.
let safe = escape("<script>alert('x')</script>");
assert_eq!(safe.as_str(), "&lt;script&gt;alert(&#39;x&#39;)&lt;/script&gt;");

// Mark trusted text safe without escaping it.
let trusted = Markup::new("<em>Hello</em>");
assert_eq!(trusted.as_str(), "<em>Hello</em>");

// Concatenation escapes only the unsafe side.
let combined = Markup::new("<em>Hello</em> ") + "<foo>";
assert_eq!(combined.as_str(), "<em>Hello</em> &lt;foo&gt;");

// Two safe pieces are joined as-is.
let both = Markup::new("<a>") + Markup::new("<b>");
assert_eq!(both.as_str(), "<a><b>");
```

## Scope

This crate covers markupsafe's core: HTML escaping and the type-safe `Markup`. Not ported:
Python's ~30 `str`-method overrides (`Markup` is a newtype here, not a `str` subclass, so it
neither has nor needs them), `%`/`format` formatting, and `unescape`/`striptags` (which need
full HTML-entity decoding; use a dedicated HTML-entities crate for that).

## License

Licensed under the [BSD 3-Clause License](LICENSE-BSD), matching the upstream `markupsafe`
project.
