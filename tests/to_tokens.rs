mod test_utils;

use quote::quote;
use structmeta::ToTokens;
use syn::parse_quote;
use test_utils::*;

#[test]
fn derive_for_struct() {
    #[derive(ToTokens)]
    struct TestStruct {
        name: syn::Ident,
        eq_token: syn::Token![=],
        value: syn::Expr,
    }

    let s = TestStruct {
        name: parse_quote!(abc),
        eq_token: parse_quote!(=),
        value: parse_quote!(1 + 2),
    };
    let ts = quote!(abc = 1 + 2);
    assert_eq_ts(s, ts);
}
#[test]
fn derive_for_tuple_struct() {
    #[derive(ToTokens)]
    struct TestStruct(syn::Ident, syn::Token![=], syn::Expr);

    let s = TestStruct(parse_quote!(abc), parse_quote!(=), parse_quote!(1 + 2));
    let ts = quote!(abc = 1 + 2);
    assert_eq_ts(s, ts);
}

#[test]
fn derive_for_unit_struct() {
    #[derive(ToTokens)]
    struct TestStruct;

    let s = TestStruct;
    let ts = quote!();
    assert_eq_ts(s, ts);
}

#[test]
fn derive_for_enum() {
    #[derive(ToTokens)]
    enum TestEnum {
        A {
            name: syn::Ident,
            eq_token: syn::Token![=],
            value: syn::Expr,
        },
        B(syn::Ident, syn::Token![=>], syn::Expr),
        C,
    }

    let s = TestEnum::A {
        name: parse_quote!(abc),
        eq_token: parse_quote!(=),
        value: parse_quote!(1 + 2),
    };
    let ts = quote!(abc = 1 + 2);
    assert_eq_ts(s, ts);

    let s = TestEnum::B(parse_quote!(abc), parse_quote!(=>), parse_quote!(1 + 2));
    let ts = quote!(abc => 1 + 2);
    assert_eq_ts(s, ts);

    let s = TestEnum::C;
    let ts = quote!();
    assert_eq_ts(s, ts);
}

#[test]
fn struct_raw_keyword_field() {
    #[derive(ToTokens)]
    struct TestStruct {
        r#mut: syn::Ident,
        eq_token: syn::Token![=],
        value: syn::Expr,
    }

    let s = TestStruct {
        r#mut: parse_quote!(abc),
        eq_token: parse_quote!(=),
        value: parse_quote!(1 + 2),
    };
    let ts = quote!(abc = 1 + 2);
    assert_eq_ts(s, ts);
}

#[test]
fn enum_raw_keyword_field() {
    #[derive(ToTokens)]
    enum TestEnum {
        A {
            r#mut: syn::Ident,
            eq_token: syn::Token![=],
            value: syn::Expr,
        },
    }

    let s = TestEnum::A {
        r#mut: parse_quote!(abc),
        eq_token: parse_quote!(=),
        value: parse_quote!(1 + 2),
    };
    let ts = quote!(abc = 1 + 2);
    assert_eq_ts(s, ts);
}

#[test]
fn brace_all() {
    #[derive(ToTokens)]
    struct TestStruct {
        #[to_tokens("{")]
        brace_token: syn::token::Brace,
        key: syn::LitStr,
        eq_token: syn::Token![=],
        value: syn::Expr,
    }

    let s = TestStruct {
        brace_token: syn::token::Brace::default(),
        key: parse_quote!("abc"),
        eq_token: parse_quote!(=),
        value: parse_quote!(1 + 2),
    };
    let ts = quote!({ "abc" = 1 + 2 });
    assert_eq_ts(s, ts);
}

#[test]
fn brace_close() {
    #[derive(ToTokens)]
    struct TestStruct {
        #[to_tokens("{")]
        brace_token: syn::token::Brace,
        key: syn::LitStr,
        eq_token: syn::Token![=],
        #[to_tokens("}")]
        value: syn::Expr,
    }

    let s = TestStruct {
        brace_token: syn::token::Brace::default(),
        key: parse_quote!("abc"),
        eq_token: parse_quote!(=),
        value: parse_quote!(1 + 2),
    };
    let ts = quote!({ "abc" = } 1 + 2 );
    assert_eq_ts(s, ts);
}
