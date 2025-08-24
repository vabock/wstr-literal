#![doc = include_str!("../README.md")]
use proc_macro::TokenStream;
use wstr_literal_impl::{wstr_impl, wstr_literal_impl};

/// Converts a string literal to a UTF-16 null-terminated array at compile time.
///
/// # Usage
///
/// - `wstr!("hello")` → `[u16; 6]` array containing UTF-16 code units + null terminator
/// - `wstr!(10, "hello")` → `[u16; 10]` array, zero-padded to the specified length
///
/// # Examples
///
/// ```rust
/// use wstr_literal::wstr;
///
/// // Basic usage
/// let greeting = wstr!("Hello");
/// assert_eq!(&greeting, &['H' as u16, 'e' as u16, 'l' as u16, 'l' as u16, 'o' as u16, 0]);
///
/// // Fixed length with padding
/// let padded = wstr!(10, "Hello");
/// assert_eq!(padded.len(), 10);
/// assert_eq!(&padded[..6], &['H' as u16, 'e' as u16, 'l' as u16, 'l' as u16, 'o' as u16, 0]); // utf16 code units + null terminator
/// assert_eq!(&padded[6..], &[0u16, 0u16, 0u16, 0u16]); // zero padding
/// ```
///
/// The following will fail to compile because the specified length is too small:
///
/// ```compile_fail
/// use wstr_literal::wstr;
/// let too_small: [u16; 3] = wstr!(3, "hello");
/// // Error: array size must be at least the length of the input string plus the null terminator
/// ```
///
/// # Requirements
///
/// - Input must be a string literal (not a variable or expression)
/// - Optional length must be an integer literal
/// - If fixed size is specified, it must be at least string length + 1(for null terminator)
///
#[proc_macro]
pub fn wstr(input: TokenStream) -> TokenStream {
    wstr_impl(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Attribute macro that transforms string literal values in `const` or `static` declarations into null-terminated UTF-16 arrays.
///
/// Apply this attribute to `const` or `static` declarations to automatically convert
/// string literals into null-terminated UTF-16 arrays at compile time.
///
/// <div class="warning">
/// This macro only works with const and static declarations. For let bindings,
/// use the wstr! macro instead.
/// </div>
///
/// # Usage
///
/// - `#[wstr_literal] const NAME: [u16; _] = "text";` → length inferred automatically
/// - `#[wstr_literal] static NAME: [u16; 10] = "text";` → fixed length with zero padding
///
/// # Examples
///
/// ```rust
/// use wstr_literal::wstr_literal;
///
/// // Length inferred from string content
/// #[wstr_literal]
/// const GREETING: [u16; _] = "Hello";
/// // Expands to: const GREETING: [u16; 6] = [72, 101, 108, 108, 111, 0];
///
/// // Fixed length with padding
/// #[wstr_literal]
/// static PADDED: [u16; 10] = "Hi";
/// // Expands to: static PADDED: [u16; 10] = [72, 105, 0, 0, 0, 0, 0, 0, 0, 0];
///
/// // Works with visibility, mutability, and other attributes
/// #[wstr_literal]
/// #[allow(non_upper_case_globals)]
/// pub static mut Global: [u16; _] = "data";
/// ```
///
/// The following will fail to compile because the array size (1) is too small
/// for the string "hello" which needs 6 elements (5 UTF-16 code units + 1 null terminator):
///
/// ```compile_fail
/// use wstr_literal::wstr_literal;
/// #[wstr_literal]
/// static HELLO: [u16; 1] = "hello";
/// // Error: array size must be at least length of input string plus null terminator
/// ```
///
/// # Requirements
///
/// - Target must be a `const` or `static` declaration
/// - Type must be `[u16; SIZE]` where `SIZE` is either `_` or a integer literal
/// - The initial value must be a string literal
/// - If fixed size is specified, it must be at least string length + 1(for null terminator)
///
#[proc_macro_attribute]
pub fn wstr_literal(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return syn::Error::new_spanned(
            proc_macro2::TokenStream::from(attr),
            "The attribute does not take any arguments",
        )
        .into_compile_error()
        .into();
    }

    wstr_literal_impl(item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
