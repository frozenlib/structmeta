mod test_utils;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use structmeta::{Parse, ToTokens};
use syn::parse::Parse;
use test_utils::*;

#[test]
fn derive_for_struct() {
    #[derive(Parse, ToTokens)]
    struct TestStruct {
        name: syn::Ident,
        eq_token: syn::Token![=],
        value: syn::Expr,
    }
    assert_parse::<TestStruct>(quote!(xxx = 1 + 2));
}

#[test]
fn derive_for_enum() {
    #[derive(ToTokens, Parse)]
    enum TestEnum {
        A(syn::Token![=], syn::Expr),
        B {
            plus_token: syn::Token![+],
            value: syn::Expr,
        },
        C,
    }
    assert_parse::<TestEnum>(quote!(= 1 + 2));
    assert_parse::<TestEnum>(quote!(+ 1 + 2));
    assert_parse::<TestEnum>(quote!());
}

fn assert_parse<T: Parse + ToTokens>(ts: TokenStream) {
    let value: T = syn::parse2(ts.clone()).expect("syn::parse2 failed.");
    assert_eq_ts(value, ts);
}
