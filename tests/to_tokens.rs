use quote::quote;
use structmeta::ToTokens;
use syn::parse_quote;

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
}

fn assert_eq_ts(s: impl quote::ToTokens, ts: proc_macro2::TokenStream) {
    let ts0 = s.to_token_stream().to_string();
    let ts1 = ts.to_string();
    assert_eq!(ts0, ts1);
}
