#![doc = include_str!("../README.md")]
use std::iter::once;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    bracketed, parse::Parse, Attribute, Ident, LitInt, LitStr, StaticMutability, Token, Visibility,
};

pub fn wstr_impl(input: TokenStream) -> syn::Result<TokenStream> {
    let WstrArgs { arr_len, input_str } = syn::parse2(input)?;

    let mut v: Vec<_> = input_str.value().encode_utf16().chain(once(0)).collect();

    if let Some(arr_len) = arr_len {
        let sz: usize = arr_len.base10_parse()?;

        if sz < v.len() {
            return Err(syn::Error::new_spanned(
                arr_len,
                "array size must be at least length of input string plus null terminator",
            ));
        }

        v.resize(sz, 0);
    }

    Ok(quote! {
        [
            #(#v),*
        ]
    })
}

struct WstrArgs {
    arr_len: Option<LitInt>,
    input_str: LitStr,
}

impl Parse for WstrArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let arr_len = if input.peek(LitInt) {
            Some(input.parse()?)
        } else {
            None
        };
        if arr_len.is_some() {
            let _: Token![,] = input.parse()?;
        }
        let input_str: LitStr = input.parse()?;
        Ok(Self { arr_len, input_str })
    }
}

fn parse_array_size(input: syn::parse::ParseStream) -> syn::Result<Option<LitInt>> {
    let lookahead = input.lookahead1();
    if lookahead.peek(LitInt) {
        Ok(Some(input.parse()?))
    } else if lookahead.peek(Token![_]) {
        let _ = input.parse::<Token![_]>()?;
        Ok(None)
    } else {
        Err(lookahead.error())
    }
}

struct WstrTypeArray {
    elem: Ident,
    size: Option<LitInt>,
}

impl Parse for WstrTypeArray {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;

        let _ = bracketed!(content in input);
        let elem: Ident = content.parse()?;
        let _ = content.parse::<Token![;]>()?;
        let size = parse_array_size(&content)?;

        Ok(Self { elem, size })
    }
}

enum WstrConstOrStatic {
    Const(Token![const]),
    Static(Token![static]),
}

impl Parse for WstrConstOrStatic {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![const]) {
            Ok(WstrConstOrStatic::Const(input.parse()?))
        } else if lookahead.peek(Token![static]) {
            Ok(WstrConstOrStatic::Static(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for WstrConstOrStatic {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            WstrConstOrStatic::Const(t) => t.to_tokens(tokens),
            WstrConstOrStatic::Static(t) => t.to_tokens(tokens),
        }
    }
}

struct WstrDeclaration {
    attrs: Vec<Attribute>,
    vis: Visibility,
    const_or_static: WstrConstOrStatic,
    mutability: StaticMutability,
    ident: Ident,
    ty: WstrTypeArray,
    lit: LitStr,
}

impl Parse for WstrDeclaration {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis: Visibility = input.parse()?;
        let const_or_static: WstrConstOrStatic = input.parse()?;
        let mutability: StaticMutability = input.parse()?;
        let ident: Ident = input.parse()?;
        let _ = input.parse::<Token![:]>()?;
        let ty: WstrTypeArray = input.parse()?;
        let _ = input.parse::<Token![=]>()?;
        let lit: LitStr = input.parse()?;
        let _ = input.parse::<Token![;]>()?;

        Ok(Self {
            attrs,
            vis,
            const_or_static,
            mutability,
            ident,
            ty,
            lit,
        })
    }
}

pub fn wstr_literal_impl(input: TokenStream) -> syn::Result<TokenStream> {
    let input = syn::parse2::<WstrDeclaration>(input)?;

    let WstrDeclaration {
        attrs,
        vis,
        const_or_static,
        mutability,
        ident,
        ty,
        lit,
    } = input;
    let WstrTypeArray { elem, size } = ty;

    let mut v: Vec<_> = lit.value().encode_utf16().chain(once(0)).collect();
    let arr_len = match size {
        Some(len) => {
            let sz: usize = len.base10_parse()?;
            if sz < v.len() {
                return Err(syn::Error::new_spanned(
                    len,
                    "array size must be at least length of input string plus null terminator",
                ));
            }
            v.resize(sz, 0);
            sz
        }
        None => v.len(),
    };

    let arr = quote! {
        [
            #(#v),*
        ]
    };

    Ok(quote! {
        #(#attrs)*
        #vis #const_or_static #mutability #ident: [#elem; #arr_len] = #arr;
    })
}

#[cfg(test)]
mod tests {
    use proc_macro2::Span;
    use syn::{ItemConst, ItemStatic};

    use super::*;

    trait RMSP {
        fn rmsp(&self) -> String;
    }

    impl RMSP for String {
        /// スペースを削除する
        fn rmsp(&self) -> String {
            self.replace(' ', "")
        }
    }

    #[test]
    fn test_wstr_impl_single_char() {
        let result = wstr_impl(quote!("A")).unwrap();
        assert_eq!(result.to_string().rmsp(), "[65u16,0u16]".to_string());
    }

    #[test]
    fn test_wstr_impl_empty_string() {
        let result = wstr_impl(quote!("")).unwrap();
        assert_eq!(result.to_string(), "[0u16]".to_string());
    }

    #[test]
    fn test_wstr_impl_ascii_string() {
        let result = wstr_impl(quote!("Hello")).unwrap();
        assert_eq!(
            result.to_string().rmsp(),
            "[72u16,101u16,108u16,108u16,111u16,0u16]".to_string()
        );
    }

    #[test]
    fn test_wstr_impl_unicode_string() {
        let result = wstr_impl(quote!("こんにちは")).unwrap();
        assert_eq!(
            result.to_string().rmsp(),
            "[12371u16,12435u16,12395u16,12385u16,12399u16,0u16]".to_string()
        );
    }

    #[test]
    fn test_wstr_impl_full_len_ascii() {
        let result = wstr_impl(quote!(10, "Hello")).unwrap();

        assert_eq!(
            result.to_string().rmsp(),
            "[72u16,101u16,108u16,108u16,111u16,0u16,0u16,0u16,0u16,0u16]".to_string()
        );
    }

    #[test]
    fn test_wstr_impl_full_len_ascii_exact_len() {
        let result = wstr_impl(quote!(6, "Hello")).unwrap();

        assert_eq!(
            result.to_string().rmsp(),
            "[72u16,101u16,108u16,108u16,111u16,0u16]".to_string()
        );
    }

    #[test]
    fn test_wstr_impl_full_len_ascii_less_len_error() {
        let result = wstr_impl(quote!(5, "Hello"));
        assert!(result.is_err());
    }

    #[test]
    fn test_wstr_impl_full_len_empty_string_exact_len() {
        // empty string + null terminator => len 1
        let result = wstr_impl(quote!(1, "")).unwrap();
        assert_eq!(result.to_string().rmsp(), "[0u16]".to_string());
    }

    #[test]
    fn test_wstr_impl_full_len_empty_string_larger_len() {
        let result = wstr_impl(quote!(5, "")).unwrap();
        assert_eq!(
            result.to_string().rmsp(),
            "[0u16,0u16,0u16,0u16,0u16]".to_string()
        );
    }

    #[test]
    fn test_wstr_impl_full_len_zero_len_error() {
        let result = wstr_impl(quote!(0, ""));
        assert!(result.is_err());
    }

    #[test]
    fn test_wstr_impl_full_len_unicode_larger_len() {
        let result = wstr_impl(quote!(10, "こんにちは")).unwrap();
        // 5 chars + null + 4 zeros padding => 10 total
        assert_eq!(
            result.to_string().rmsp(),
            "[12371u16,12435u16,12395u16,12385u16,12399u16,0u16,0u16,0u16,0u16,0u16]".to_string()
        );
    }

    #[test]
    fn test_wstr_impl_full_len_unicode_exact_len() {
        let result = wstr_impl(quote!(6, "こんにちは")).unwrap();
        assert_eq!(
            result.to_string().rmsp(),
            "[12371u16,12435u16,12395u16,12385u16,12399u16,0u16]".to_string()
        );
    }

    #[test]
    fn test_wstr_literal_impl_static_placeholder() {
        let input = quote! {
            static HELLO: [u16; _] = "hello";
        };
        let output = wstr_literal_impl(input).unwrap();
        let item = syn::parse2::<ItemStatic>(output).unwrap();

        assert_eq!(
            item.ty.to_token_stream().to_string().rmsp(),
            "[u16;6usize]".to_string()
        );
        assert_eq!(
            item.expr.to_token_stream().to_string().rmsp(),
            "[104u16,101u16,108u16,108u16,111u16,0u16]".to_string()
        );
    }

    #[test]
    fn test_wstr_literal_impl_static_larger_len() {
        let input = quote! {
            static HELLO: [u16; 10] = "hello";
        };
        let output = wstr_literal_impl(input).unwrap();
        let item = syn::parse2::<ItemStatic>(output).unwrap();

        assert_eq!(
            item.ty.to_token_stream().to_string().rmsp(),
            "[u16;10usize]".to_string()
        );
        assert_eq!(
            item.expr.to_token_stream().to_string().rmsp(),
            "[104u16,101u16,108u16,108u16,111u16,0u16,0u16,0u16,0u16,0u16]".to_string()
        );
    }

    #[test]
    fn test_wstr_literal_impl_const_placeholder() {
        let input = quote! {
            const HELLO: [u16; _] = "hello";
        };
        let output = wstr_literal_impl(input).unwrap();
        let item = syn::parse2::<ItemConst>(output).unwrap();

        assert_eq!(
            item.ty.to_token_stream().to_string().rmsp(),
            "[u16;6usize]".to_string()
        );
        assert_eq!(
            item.expr.to_token_stream().to_string().rmsp(),
            "[104u16,101u16,108u16,108u16,111u16,0u16]".to_string()
        );
    }

    #[test]
    fn test_wstr_literal_impl_const_larger_len() {
        let input = quote! {
            const HELLO: [u16; 10] = "hello";
        };
        let output = wstr_literal_impl(input).unwrap();
        let item = syn::parse2::<ItemConst>(output).unwrap();

        assert_eq!(
            item.ty.to_token_stream().to_string().rmsp(),
            "[u16;10usize]".to_string()
        );
        assert_eq!(
            item.expr.to_token_stream().to_string().rmsp(),
            "[104u16,101u16,108u16,108u16,111u16,0u16,0u16,0u16,0u16,0u16]".to_string()
        );
    }

    #[test]
    fn test_wstr_typearray() {
        let output = syn::parse2::<WstrTypeArray>(quote!([u16; _])).unwrap();
        assert_eq!(output.elem, Ident::new("u16", Span::call_site()));
        assert!(matches!(output.size, None));

        let output = syn::parse2::<WstrTypeArray>(quote!([u16; 0x10])).unwrap();
        assert_eq!(output.elem, Ident::new("u16", Span::call_site()));
        assert!(matches!(output.size, Some(_)));
        assert_eq!(
            match output.size {
                Some(size) => size.base10_parse::<usize>().unwrap(),
                _ => unreachable!(),
            },
            0x10
        );
    }
}
