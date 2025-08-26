[![Crates.io][crates-badge]][crates-url]
[![docs.rs][docs_rs-badge]][docs_rs-url]
[![Github Actions][ci-badge]][ci-url]

[crates-badge]: https://img.shields.io/crates/v/wstr-literal.svg
[crates-url]: https://crates.io/crates/wstr-literal
[docs_rs-badge]: https://img.shields.io/docsrs/wstr-literal.svg
[docs_rs-url]: https://docs.rs/wstr-literal
[ci-badge]: https://github.com/vabock/wstr-literal/actions/workflows/test.yml/badge.svg?branch=main
[ci-url]: https://github.com/vabock/wstr-literal/actions?query=branch%3Amain

Procedural macros for building UTF-16 null-terminated string arrays for Windows FFI and similar APIs at compile time.

This crate provides two macros:

- [`wstr!`] — a function-like macro that converts a string literal to a UTF-16 array with a trailing null (0u16). Optionally accepts an explicit array length and pads with zeros.
- #\[[`wstr_literal`]\] — an attribute macro for const or static declaration that transforms a string-literal into a UTF-16 array with a trailing null. Supports either a fixed array length or an inferred length via the `_` placeholder.

Both macros expand at compile time into numeric `[u16; N]` array literals; there is no runtime allocation or conversion.

# Installation

Add this crate to your `Cargo.toml`:

```toml
[dependencies]
wstr-literal = "0.1"
```

# Examples:

```rust
use wstr_literal::{wstr_literal, wstr};

// Exact-length array (5 code units + 1 null terminator)
let hello: [u16; 6] = wstr!("Hello");
assert_eq!(hello, [
    'H' as u16, 'e' as u16, 'l' as u16, 'l' as u16, 'o' as u16, 0
]);

// Pad to a larger, fixed length
let padded: [u16; 10] = wstr!(10, "hello");
assert_eq!(&padded[..6], &[0x68, 0x65, 0x6c, 0x6c, 0x6f, 0]);
assert_eq!(&padded[6..], &[0u16; 4]);

// Slice
let slice: &[u16] = &wstr!("Hi");
assert_eq!(slice, &[0x48, 0x69, 0]);

#[wstr_literal]
const HELLO_AUTO: [u16; _] = "hello"; // length inferred (5 + 1)
assert_eq!(HELLO_AUTO.len(), 6);

#[wstr_literal]
static EMOJI: [u16; _] = "\u{1F917}"; // 🤗
assert_eq!(EMOJI.len(), 3);

#[wstr_literal]
static HELLO_FIXED: [u16; 0x10] = "hello"; // padded with zeros to length 0x10
assert_eq!(&HELLO_FIXED[..6], &[0x68, 0x65, 0x6c, 0x6c, 0x6f, 0]);
assert_eq!(&HELLO_FIXED[6..], &[0u16; 10]); // padding

#[wstr_literal]
#[allow(non_upper_case_globals)]
pub static mut GlobalMut: [u16; _] = "x"; // supports const/static, visibility, mutability, and other attributes
```

# Using with [windows-rs](https://crates.io/crates/windows)

These arrays are suitable for FFI calls expecting UTF-16 with a trailing null, for example with the windows crate:

```rust,ignore
use wstr_literal::wstr_literal;
use windows::core::PCWSTR;

#[wstr_literal]
static APP_NAME: [u16; _] = "MyApp";

let pcwstr = PCWSTR(APP_NAME.as_ptr());
// pass `pcwstr` to Windows APIs that expect a wide string
```

# Acknowledgments

This macro was inspired by the [`auto-const-array`](https://crates.io/crates/auto-const-array) crate.
