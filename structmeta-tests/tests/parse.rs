mod test_utils;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use structmeta::{Parse, ToTokens};
use syn::{parse::Parse, punctuated::Punctuated, token, Expr, LitInt, LitStr, Token};
use syn::{Ident, MacroDelimiter};
use test_utils::*;

#[test]
fn for_tuple() {
    #[derive(Parse, ToTokens)]
    struct TestTuple(LitInt, LitStr);
    assert_parse::<TestTuple>(quote!(10 "abc"));
}

#[test]
fn for_struct() {
    #[derive(Parse, ToTokens)]
    struct TestStruct {
        name: Ident,
        eq_token: Token![=],
        value: Expr,
    }
    assert_parse::<TestStruct>(quote!(xxx = 1 + 2));
}

#[test]
fn for_enum() {
    #[allow(clippy::large_enum_variant)]
    #[derive(ToTokens, Parse)]
    enum TestEnum {
        A(Token![=], LitInt, LitInt),
        B { plus_token: Token![+], value: Expr },
        C,
    }
    assert_parse::<TestEnum>(quote!(= 1 2));
    assert_parse::<TestEnum>(quote!(+ 1 + 2));
    assert_parse::<TestEnum>(quote!());

    assert_parse_fail::<TestEnum>(quote!(= 1));
}

#[test]
fn brace_all() {
    #[derive(Parse, ToTokens)]
    struct TestStruct {
        #[to_tokens("{")]
        brace_token: token::Brace,
        key: syn::LitStr,
        eq_token: Token![=],
        value: Expr,
    }

    assert_parse::<TestStruct>(quote!({ "abc" = 1 + 2 }));
}

#[test]
fn brace_close() {
    #[derive(Parse, ToTokens)]
    struct TestStruct {
        #[to_tokens("{")]
        brace_token: token::Brace,
        key: syn::LitStr,
        eq_token: Token![=],
        #[to_tokens("}")]
        value: Expr,
    }
    assert_parse::<TestStruct>(quote!({ "abc" = } 1 + 2));
}

#[test]
fn paren_all() {
    #[derive(Parse, ToTokens)]
    struct TestStruct {
        #[to_tokens("(")]
        paren_token: token::Paren,
        key: syn::LitStr,
        eq_token: Token![=],
        value: Expr,
    }
    assert_parse::<TestStruct>(quote!(("abc" = 1 + 2)));
}

#[test]
fn paren_close() {
    #[derive(Parse, ToTokens)]
    struct TestStruct {
        #[to_tokens("(")]
        brace_token: token::Paren,
        key: syn::LitStr,
        eq_token: Token![=],
        #[to_tokens(")")]
        value: Expr,
    }
    assert_parse::<TestStruct>(quote!(("abc" = ) 1 + 2 ));
}

#[test]
fn paren_nested() {
    #[derive(Parse, ToTokens)]
    struct TestStruct {
        #[to_tokens("(")]
        brace_token1: token::Paren,
        key: syn::LitStr,

        #[to_tokens("(")]
        brace_token2: token::Paren,

        eq_token: Token![=],
        #[to_tokens(")")]
        value: Expr,
    }
    assert_parse::<TestStruct>(quote!(("abc" ( = ) 1 + 2 )));
}

#[test]
fn paren_close_many() {
    #[derive(Parse, ToTokens)]
    struct TestStruct {
        #[to_tokens("(")]
        brace_token1: token::Paren,
        key: syn::LitStr,

        #[to_tokens("(")]
        brace_token2: token::Paren,

        eq_token: Token![=],
        #[to_tokens("))")]
        value: Expr,
    }
    assert_parse::<TestStruct>(quote!(("abc" ( = )) 1 + 2 ));
}

#[test]
fn paren_close_open() {
    #[derive(Parse, ToTokens)]
    struct TestStruct {
        #[to_tokens("(")]
        brace_token1: token::Paren,
        key: syn::LitStr,

        #[to_tokens(")(")]
        brace_token2: token::Paren,

        eq_token: Token![=],
        value: Expr,
    }
    assert_parse::<TestStruct>(quote!(("abc")( = 1 + 2 )));
}

#[test]
fn bracket_all() {
    #[derive(Parse, ToTokens)]
    struct TestStruct {
        #[to_tokens("[")]
        paren_token: token::Bracket,
        key: syn::LitStr,
        eq_token: Token![=],
        value: Expr,
    }
    assert_parse::<TestStruct>(quote!(["abc" = 1 + 2]));
}

#[test]
fn bracket_close() {
    #[derive(Parse, ToTokens)]
    struct TestStruct {
        #[to_tokens("[")]
        brace_token: token::Bracket,
        key: syn::LitStr,
        eq_token: Token![=],
        #[to_tokens("]")]
        value: Expr,
    }
    assert_parse::<TestStruct>(quote!(["abc" = ] 1 + 2 ));
}

#[test]
fn macro_delimiter_all() {
    #[derive(Parse, ToTokens)]
    struct TestStruct {
        #[to_tokens("(")]
        paren_token: MacroDelimiter,
        key: syn::LitStr,
        eq_token: Token![=],
        value: Expr,
    }
    assert_parse::<TestStruct>(quote!(["abc" = 1 + 2]));
    assert_parse::<TestStruct>(quote!(("abc" = 1 + 2)));
    assert_parse::<TestStruct>(quote!({ "abc" = 1 + 2 }));
}

#[test]
fn macro_delimiter_close() {
    #[derive(Parse, ToTokens)]
    struct TestStruct {
        #[to_tokens("(")]
        brace_token: MacroDelimiter,
        key: syn::LitStr,
        eq_token: Token![=],
        #[to_tokens(")")]
        value: Expr,
    }
    assert_parse::<TestStruct>(quote!(["abc" = ] 1 + 2 ));
    assert_parse::<TestStruct>(quote!(("abc" = ) 1 + 2 ));
    assert_parse::<TestStruct>(quote!({"abc" = } 1 + 2 ));
}

#[test]
fn peek() {
    #[derive(Parse, ToTokens)]
    enum TestEnum {
        A {
            #[parse(peek)]
            eq_token: Token![=],
        },
        B {
            #[parse(peek)]
            plus_token: Token![+],
        },
    }

    assert_parse::<TestEnum>(quote!(=));
    assert_parse::<TestEnum>(quote!(+));
}

#[test]
fn peek_no_fall_through() {
    #[derive(Parse, ToTokens)]
    enum TestEnum {
        A(#[parse(peek)] Token![=], LitStr),
        B(Token![=], LitInt),
    }

    assert_parse::<TestEnum>(quote!(= "abc"));
    assert_parse_fail::<TestEnum>(quote!(= 1));
}

#[test]
fn peek2() {
    #[derive(Parse, ToTokens)]
    enum TestEnum {
        A {
            #[parse(peek)]
            key: Ident,
            #[parse(peek)]
            eq_token: Token![=],
        },
        B {
            #[parse(peek)]
            key: Ident,
            #[parse(peek)]
            plus_token: Token![+],
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
            key: Ident,
            #[parse(peek)]
            eq_token: Token![=],
            #[parse(peek)]
            value: Ident,
        },
        B {
            #[parse(peek)]
            key: Ident,
            #[parse(peek)]
            plus_token: Token![+],
            #[parse(peek)]
            value: Ident,
        },
    }

    assert_parse::<TestEnum>(quote!(a = x));
    assert_parse::<TestEnum>(quote!(a + y));
}

#[test]
fn parse_any() {
    #[derive(Parse, ToTokens)]
    struct TestStruct {
        #[parse(any)]
        key: Ident,
        eq_token: Token![=],
    }
    assert_parse::<TestStruct>(quote!(struct =));
}

#[test]
fn peek_bracket() {
    #[derive(Parse, ToTokens)]
    enum TestEnum {
        A {
            #[parse(peek)]
            #[to_tokens("[")]
            bracket_token: token::Bracket,
            eq_token: Token![=],
            #[to_tokens("]")]
            #[parse(peek)]
            name: Ident,
        },
    }
    assert_parse::<TestEnum>(quote!([=] abc));
}

#[test]
fn peek_any() {
    #[derive(Parse, ToTokens)]
    enum TestEnum {
        A {
            #[parse(peek, any)]
            key: Ident,
            #[parse(peek)]
            eq_token: Token![=],
        },
    }
    assert_parse::<TestEnum>(quote!(struct =));
}

#[test]
fn parse_terminated() {
    #[derive(Parse, ToTokens)]
    struct TestStruct {
        #[parse(terminated)]
        key: Punctuated<LitStr, Token![,]>,
    }
    assert_parse::<TestStruct>(quote!("a"));
    assert_parse::<TestStruct>(quote!("a",));
    assert_parse::<TestStruct>(quote!("a", "b"));
}

#[test]
fn parse_terminated_any() {
    #[derive(Parse, ToTokens)]
    struct TestStruct {
        #[parse(terminated, any)]
        key: Punctuated<Ident, Token![,]>,
    }
    assert_parse::<TestStruct>(quote!(a));
    assert_parse::<TestStruct>(quote!(a,));
    assert_parse::<TestStruct>(quote!(a, b, struct));
}

#[track_caller]
fn assert_parse<T: Parse + ToTokens>(ts: TokenStream) {
    let value: T = syn::parse2(ts.clone()).expect("syn::parse2 failed.");
    assert_eq_ts(value, ts);
}

#[track_caller]
fn assert_parse_fail<T: Parse + ToTokens>(ts: TokenStream) {
    let value: syn::Result<T> = syn::parse2(ts);
    if value.is_ok() {
        panic!("expect parse failed, but parse succeeded.");
    }
}
