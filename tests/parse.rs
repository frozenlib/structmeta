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
#[test]
fn brace_all() {
    #[derive(Parse, ToTokens)]
    struct TestStruct {
        #[to_tokens("{")]
        brace_token: syn::token::Brace,
        key: syn::LitStr,
        eq_token: syn::Token![=],
        value: syn::Expr,
    }

    assert_parse::<TestStruct>(quote!({ "abc" = 1 + 2 }));
}

#[test]
fn brace_close() {
    #[derive(Parse, ToTokens)]
    struct TestStruct {
        #[to_tokens("{")]
        brace_token: syn::token::Brace,
        key: syn::LitStr,
        eq_token: syn::Token![=],
        #[to_tokens("}")]
        value: syn::Expr,
    }
    assert_parse::<TestStruct>(quote!({ "abc" = } 1 + 2));
}

#[test]
fn paren_all() {
    #[derive(Parse, ToTokens)]
    struct TestStruct {
        #[to_tokens("(")]
        paren_token: syn::token::Paren,
        key: syn::LitStr,
        eq_token: syn::Token![=],
        value: syn::Expr,
    }
    assert_parse::<TestStruct>(quote!(("abc" = 1 + 2)));
}

#[test]
fn paren_close() {
    #[derive(Parse, ToTokens)]
    struct TestStruct {
        #[to_tokens("(")]
        brace_token: syn::token::Paren,
        key: syn::LitStr,
        eq_token: syn::Token![=],
        #[to_tokens(")")]
        value: syn::Expr,
    }
    assert_parse::<TestStruct>(quote!(("abc" = ) 1 + 2 ));
}

#[test]
fn paren_nested() {
    #[derive(Parse, ToTokens)]
    struct TestStruct {
        #[to_tokens("(")]
        brace_token1: syn::token::Paren,
        key: syn::LitStr,

        #[to_tokens("(")]
        brace_token2: syn::token::Paren,

        eq_token: syn::Token![=],
        #[to_tokens(")")]
        value: syn::Expr,
    }
    assert_parse::<TestStruct>(quote!(("abc" ( = ) 1 + 2 )));
}

#[test]
fn paren_close_many() {
    #[derive(Parse, ToTokens)]
    struct TestStruct {
        #[to_tokens("(")]
        brace_token1: syn::token::Paren,
        key: syn::LitStr,

        #[to_tokens("(")]
        brace_token2: syn::token::Paren,

        eq_token: syn::Token![=],
        #[to_tokens("))")]
        value: syn::Expr,
    }
    assert_parse::<TestStruct>(quote!(("abc" ( = )) 1 + 2 ));
}

#[test]
fn paren_close_open() {
    #[derive(Parse, ToTokens)]
    struct TestStruct {
        #[to_tokens("(")]
        brace_token1: syn::token::Paren,
        key: syn::LitStr,

        #[to_tokens(")(")]
        brace_token2: syn::token::Paren,

        eq_token: syn::Token![=],
        value: syn::Expr,
    }
    assert_parse::<TestStruct>(quote!(("abc")( = 1 + 2 )));
}

#[test]
fn bracket_all() {
    #[derive(Parse, ToTokens)]
    struct TestStruct {
        #[to_tokens("[")]
        paren_token: syn::token::Bracket,
        key: syn::LitStr,
        eq_token: syn::Token![=],
        value: syn::Expr,
    }
    assert_parse::<TestStruct>(quote!(["abc" = 1 + 2]));
}

#[test]
fn bracket_close() {
    #[derive(Parse, ToTokens)]
    struct TestStruct {
        #[to_tokens("[")]
        brace_token: syn::token::Bracket,
        key: syn::LitStr,
        eq_token: syn::Token![=],
        #[to_tokens("]")]
        value: syn::Expr,
    }
    assert_parse::<TestStruct>(quote!(["abc" = ] 1 + 2 ));
}

#[test]
fn peek() {
    #[derive(Parse, ToTokens)]
    enum TestEnum {
        A {
            #[parse(peek)]
            eq_token: syn::Token![=],
        },
        B {
            #[parse(peek)]
            plus_token: syn::Token![+],
        },
    }

    assert_parse::<TestEnum>(quote!(=));
    assert_parse::<TestEnum>(quote!(+));
}

#[test]
fn peek2() {
    #[derive(Parse, ToTokens)]
    enum TestEnum {
        A {
            #[parse(peek)]
            key: syn::Ident,
            #[parse(peek)]
            eq_token: syn::Token![=],
        },
        B {
            #[parse(peek)]
            key: syn::Ident,
            #[parse(peek)]
            plus_token: syn::Token![+],
        },
    }

    assert_parse::<TestEnum>(quote!(a=));
    assert_parse::<TestEnum>(quote!(a+));
}

#[test]
fn peek3() {
    #[derive(Parse, ToTokens)]
    enum TestEnum {
        A {
            #[parse(peek)]
            key: syn::Ident,
            #[parse(peek)]
            eq_token: syn::Token![=],
            #[parse(peek)]
            value: syn::Ident,
        },
        B {
            #[parse(peek)]
            key: syn::Ident,
            #[parse(peek)]
            plus_token: syn::Token![+],
            #[parse(peek)]
            value: syn::Ident,
        },
    }

    assert_parse::<TestEnum>(quote!(a = x));
    assert_parse::<TestEnum>(quote!(a + y));
}

// #[test]
// fn peek_any() {
//     use syn::ext::IdentExt;
//     use syn::Ident;
//     #[derive(Parse, ToTokens)]
//     enum TestEnum {
//         A {
//             #[parse(peek(Ident::peek_any))]
//             key: syn::Ident,
//             #[parse(peek)]
//             eq_token: syn::Token![=],
//         },
//     }
//     assert_parse::<TestEnum>(quote!(struct =));
// }

fn assert_parse<T: Parse + ToTokens>(ts: TokenStream) {
    let value: T = syn::parse2(ts.clone()).expect("syn::parse2 failed.");
    assert_eq_ts(value, ts);
}
