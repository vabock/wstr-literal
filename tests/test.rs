use std::iter::{once, repeat};

use wstr_literal::{wstr, wstr_literal};

#[wstr_literal]
static WSTR_HELLO: [u16; _] = "hello";
static HELLO: [u16; 6] = [0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x0];

//#[test]
#[allow(dead_code)]
fn test_hello() {
    print!("[ ");
    "hello".encode_utf16().chain(once(0)).for_each(|c| {
        print!("{:#x}, ", c);
    });
    println!(" ]");
}

#[test]
fn test1() {
    let expected = [
        'H' as u16, 'e' as u16, 'l' as u16, 'l' as u16, 'o' as u16, 0u16,
    ];
    let result: [u16; 6] = wstr!("Hello");

    println!("{:?}", result);

    assert_eq!(result, expected);
}

#[test]
fn test_wstr_with_len() {
    let result: &[u16] = &wstr!(10, "hello");

    let expected = HELLO
        .into_iter()
        .chain(repeat(0u16))
        .take(10)
        .collect::<Vec<_>>();

    assert_eq!(result, expected.as_slice());
}

#[wstr_literal]
pub static GLOBAL_STATIC_DEFINED: [u16; _] = "a";

mod global {
    use wstr_literal::wstr_literal;
    #[wstr_literal]
    #[allow(non_upper_case_globals)]
    pub static global: [u16; _] = "data";
}

#[test]
fn test3() {
    const EXPECTED: [u16; 2] = ['a' as u16, 0];
    #[wstr_literal]
    const CONST_DEFINED: [u16; _] = "a";
    #[wstr_literal]
    static STATIC_DEFINED: [u16; _] = "a";

    assert_eq!(CONST_DEFINED, EXPECTED);
    assert_eq!(STATIC_DEFINED, EXPECTED);
    assert_eq!(GLOBAL_STATIC_DEFINED, EXPECTED);
}
