mod test_utils;

use proc_macro2::TokenStream;
use quote::quote;
use structmeta::ToTokens;
use syn::parse_quote;
use test_utils::*;

#[test]
fn for_struct() {
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
fn for_tuple_struct() {
    #[derive(ToTokens)]
    struct TestStruct(syn::Ident, syn::Token![=], syn::Expr);

    let s = TestStruct(parse_quote!(abc), parse_quote!(=), parse_quote!(1 + 2));
    let ts = quote!(abc = 1 + 2);
    assert_eq_ts(s, ts);
}

#[test]
fn for_unit_struct() {
    #[derive(ToTokens)]
    struct TestStruct;

    let s = TestStruct;
    let ts = quote!();
    assert_eq_ts(s, ts);
}

#[test]
fn for_enum() {
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
        brace_token: Default::default(),
        key: parse_quote!("abc"),
        eq_token: parse_quote!(=),
        value: parse_quote!(1 + 2),
    };
    let ts = quote!({ "abc" = } 1 + 2 );
    assert_eq_ts(s, ts);
}

#[test]
fn paren_all() {
    #[derive(ToTokens)]
    struct TestStruct {
        #[to_tokens("(")]
        paren_token: syn::token::Paren,
        key: syn::LitStr,
        eq_token: syn::Token![=],
        value: syn::Expr,
    }

    let s = TestStruct {
        paren_token: Default::default(),
        key: parse_quote!("abc"),
        eq_token: parse_quote!(=),
        value: parse_quote!(1 + 2),
    };
    let ts = quote!(("abc" = 1 + 2));
    assert_eq_ts(s, ts);
}

#[test]
fn paren_close() {
    #[derive(ToTokens)]
    struct TestStruct {
        #[to_tokens("(")]
        paren_token: syn::token::Paren,
        key: syn::LitStr,
        eq_token: syn::Token![=],
        #[to_tokens(")")]
        value: syn::Expr,
    }

    let s = TestStruct {
        paren_token: Default::default(),
        key: parse_quote!("abc"),
        eq_token: parse_quote!(=),
        value: parse_quote!(1 + 2),
    };
    let ts = quote!(("abc" = ) 1 + 2 );
    assert_eq_ts(s, ts);
}

#[test]
fn paren_nested() {
    #[derive(ToTokens)]
    struct TestStruct {
        #[to_tokens("(")]
        paren_token1: syn::token::Paren,
        key: syn::LitStr,

        #[to_tokens("(")]
        paren_token2: syn::token::Paren,

        eq_token: syn::Token![=],
        #[to_tokens(")")]
        value: syn::Expr,
    }

    let s = TestStruct {
        paren_token1: Default::default(),
        key: parse_quote!("abc"),
        paren_token2: Default::default(),
        eq_token: parse_quote!(=),
        value: parse_quote!(1 + 2),
    };
    let ts = quote!(("abc" ( = ) 1 + 2 ));
    assert_eq_ts(s, ts);
}

#[test]
fn paren_close_many() {
    #[derive(ToTokens)]
    struct TestStruct {
        #[to_tokens("(")]
        paren_token1: syn::token::Paren,
        key: syn::LitStr,

        #[to_tokens("(")]
        paren_token2: syn::token::Paren,

        eq_token: syn::Token![=],
        #[to_tokens("))")]
        value: syn::Expr,
    }

    let s = TestStruct {
        paren_token1: Default::default(),
        key: parse_quote!("abc"),
        paren_token2: Default::default(),
        eq_token: parse_quote!(=),
        value: parse_quote!(1 + 2),
    };
    let ts = quote!(("abc" ( = )) 1 + 2 );
    assert_eq_ts(s, ts);
}

#[test]
fn paren_close_open() {
    #[derive(ToTokens)]
    struct TestStruct {
        #[to_tokens("(")]
        paren_token1: syn::token::Paren,
        key: syn::LitStr,

        #[to_tokens(")(")]
        paren_token2: syn::token::Paren,

        eq_token: syn::Token![=],
        value: syn::Expr,
    }

    let s = TestStruct {
        paren_token1: Default::default(),
        key: parse_quote!("abc"),
        paren_token2: Default::default(),
        eq_token: parse_quote!(=),
        value: parse_quote!(1 + 2),
    };
    let ts = quote!(("abc")( = 1 + 2 ));
    assert_eq_ts(s, ts);
}

#[test]
fn bracket_all() {
    #[derive(ToTokens)]
    struct TestStruct {
        #[to_tokens("[")]
        braket_token: syn::token::Bracket,
        key: syn::LitStr,
        eq_token: syn::Token![=],
        value: syn::Expr,
    }

    let s = TestStruct {
        braket_token: Default::default(),
        key: parse_quote!("abc"),
        eq_token: parse_quote!(=),
        value: parse_quote!(1 + 2),
    };
    let ts = quote!(["abc" = 1 + 2]);
    assert_eq_ts(s, ts);
}

#[test]
fn bracket_close() {
    #[derive(ToTokens)]
    struct TestStruct {
        #[to_tokens("[")]
        braket_token: syn::token::Bracket,
        key: syn::LitStr,
        eq_token: syn::Token![=],
        #[to_tokens("]")]
        value: syn::Expr,
    }

    let s = TestStruct {
        braket_token: Default::default(),
        key: parse_quote!("abc"),
        eq_token: parse_quote!(=),
        value: parse_quote!(1 + 2),
    };
    let ts = quote!(["abc" = ] 1 + 2 );
    assert_eq_ts(s, ts);
}

#[test]
fn macro_delimiter_all() {
    #[derive(ToTokens)]
    struct TestStructParen {
        #[to_tokens("(")]
        delimiter: syn::MacroDelimiter,
        key: syn::LitStr,
        eq_token: syn::Token![=],
        value: syn::Expr,
    }
    #[derive(ToTokens)]
    struct TestStructBrace {
        #[to_tokens("{")]
        delimiter: syn::MacroDelimiter,
        key: syn::LitStr,
        eq_token: syn::Token![=],
        value: syn::Expr,
    }
    #[derive(ToTokens)]
    struct TestStructBraket {
        #[to_tokens("[")]
        delimiter: syn::MacroDelimiter,
        key: syn::LitStr,
        eq_token: syn::Token![=],
        value: syn::Expr,
    }

    fn check(delimiter: syn::MacroDelimiter, ts: TokenStream) {
        let s = TestStructParen {
            delimiter: delimiter.clone(),
            key: parse_quote!("abc"),
            eq_token: parse_quote!(=),
            value: parse_quote!(1 + 2),
        };
        assert_eq_ts(s, ts.clone());

        let s = TestStructBrace {
            delimiter: delimiter.clone(),
            key: parse_quote!("abc"),
            eq_token: parse_quote!(=),
            value: parse_quote!(1 + 2),
        };
        assert_eq_ts(s, ts.clone());

        let s = TestStructBraket {
            delimiter,
            key: parse_quote!("abc"),
            eq_token: parse_quote!(=),
            value: parse_quote!(1 + 2),
        };
        assert_eq_ts(s, ts);
    }
    check(
        syn::MacroDelimiter::Paren(Default::default()),
        quote!(("abc" = 1 + 2)),
    );
    check(
        syn::MacroDelimiter::Brace(Default::default()),
        quote!({ "abc" = 1 + 2 }),
    );
    check(
        syn::MacroDelimiter::Bracket(Default::default()),
        quote!(["abc" = 1 + 2]),
    );
}

#[test]
fn macro_delimiter_close() {
    #[derive(ToTokens)]
    struct TestStructParen {
        #[to_tokens("(")]
        delimiter: syn::MacroDelimiter,
        key: syn::LitStr,
        eq_token: syn::Token![=],
        #[to_tokens(")")]
        value: syn::Expr,
    }
    #[derive(ToTokens)]
    struct TestStructBrace {
        #[to_tokens("{")]
        delimiter: syn::MacroDelimiter,
        key: syn::LitStr,
        eq_token: syn::Token![=],
        #[to_tokens("}")]
        value: syn::Expr,
    }
    #[derive(ToTokens)]
    struct TestStructBraket {
        #[to_tokens("[")]
        delimiter: syn::MacroDelimiter,
        key: syn::LitStr,
        eq_token: syn::Token![=],
        #[to_tokens("]")]
        value: syn::Expr,
    }
    fn check(delimiter: syn::MacroDelimiter, ts: TokenStream) {
        let s = TestStructParen {
            delimiter: delimiter.clone(),
            key: parse_quote!("abc"),
            eq_token: parse_quote!(=),
            value: parse_quote!(1 + 2),
        };
        assert_eq_ts(s, ts.clone());

        let s = TestStructBrace {
            delimiter: delimiter.clone(),
            key: parse_quote!("abc"),
            eq_token: parse_quote!(=),
            value: parse_quote!(1 + 2),
        };
        assert_eq_ts(s, ts.clone());

        let s = TestStructBraket {
            delimiter,
            key: parse_quote!("abc"),
            eq_token: parse_quote!(=),
            value: parse_quote!(1 + 2),
        };
        assert_eq_ts(s, ts);
    }
    check(
        syn::MacroDelimiter::Paren(Default::default()),
        quote!(("abc" = ) 1 + 2),
    );
    check(
        syn::MacroDelimiter::Brace(Default::default()),
        quote!({ "abc" = } 1 + 2),
    );
    check(
        syn::MacroDelimiter::Bracket(Default::default()),
        quote!(["abc" = ] 1 + 2),
    );
}
