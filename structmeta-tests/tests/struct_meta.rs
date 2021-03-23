use proc_macro2::Span;
use std::fmt::Debug;
use structmeta::*;
use syn::{parse::Parse, parse_quote, Attribute, LitInt, LitStr};

macro_rules! pq {
    ($($tt:tt)*) =>  { parse_quote!($($tt)*) }
}

#[test]
fn test_unit() {
    #[derive(StructMeta, PartialEq, Debug)]
    struct Attr;
    check(pq!(#[attr()]), Attr);
}

#[test]
fn test_tuple_field1() {
    #[derive(StructMeta, PartialEq, Debug)]
    struct Attr(LitStr);
    check(pq!(#[attr("abc")]), Attr(pq!("abc")));
}

#[test]
fn test_tuple_field2() {
    #[derive(StructMeta, PartialEq, Debug)]
    struct Attr(LitStr, LitInt);
    check(pq!(#[attr("abc", 12)]), Attr(pq!("abc"), pq!(12)));
}

#[test]
fn test_tuple_optional_1() {
    #[derive(StructMeta, PartialEq, Debug)]
    struct Attr(Option<LitStr>);
    check(pq!(#[attr("abc")]), Attr(Some(pq!("abc"))));
    check(pq!(#[attr()]), Attr(None));
}

#[test]
fn test_tuple_optional_2() {
    #[derive(StructMeta, PartialEq, Debug)]
    struct Attr(Option<LitStr>, Option<LitInt>);
    check_msg(
        pq!(#[attr("abc", 12)]),
        Attr(Some(pq!("abc")), Some(pq!(12))),
        "args 2",
    );
    check_msg(pq!(#[attr("abc")]), Attr(Some(pq!("abc")), None), "args 1");
    check_msg(pq!(#[attr()]), Attr(None, None), "args_0");
}

#[test]
fn test_tuple_required_and_optional() {
    #[derive(StructMeta, PartialEq, Debug)]
    struct Attr(LitStr, Option<LitInt>);
    check_msg(
        pq!(#[attr("abc", 12)]),
        Attr(pq!("abc"), Some(pq!(12))),
        "args 2",
    );
    check_msg(pq!(#[attr("abc")]), Attr(pq!("abc"), None), "args 1");
}
#[test]
fn test_tuple_variadic() {
    #[derive(StructMeta, PartialEq, Debug)]
    struct Attr(Vec<LitInt>);

    check_msg(pq!(#[attr()]), Attr(vec![]), "args 0");
    check_msg(pq!(#[attr(0)]), Attr(vec![pq!(0)]), "args 1");
    check_msg(pq!(#[attr(0, 1)]), Attr(vec![pq!(0), pq!(1)]), "args 2");
}

#[test]
fn test_tuple_required_and_variadic() {
    #[derive(StructMeta, PartialEq, Debug)]
    struct Attr(LitStr, Vec<LitInt>);

    check_msg(pq!(#[attr("abc")]), Attr(pq!("abc"), vec![]), "args 1");
    check_msg(
        pq!(#[attr("abc", 0)]),
        Attr(pq!("abc"), vec![pq!(0)]),
        "args 2",
    );
    check_msg(
        pq!(#[attr("abc", 0, 1)]),
        Attr(pq!("abc"), vec![pq!(0), pq!(1)]),
        "args 3",
    );
}

#[test]
fn test_tuple_optional_and_variadic() {
    #[derive(StructMeta, PartialEq, Debug)]
    struct Attr(Option<LitStr>, Vec<LitInt>);

    check_msg(pq!(#[attr()]), Attr(None, vec![]), "args 0");
    check_msg(
        pq!(#[attr("abc")]),
        Attr(Some(pq!("abc")), vec![]),
        "args 1",
    );
    check_msg(
        pq!(#[attr("abc", 0)]),
        Attr(Some(pq!("abc")), vec![pq!(0)]),
        "args 2",
    );
    check_msg(
        pq!(#[attr("abc", 0, 1)]),
        Attr(Some(pq!("abc")), vec![pq!(0), pq!(1)]),
        "args 3",
    );
}
#[test]
fn test_tuple_requied_named() {
    #[derive(StructMeta, PartialEq, Debug)]
    struct Attr(#[name("abc")] LitStr);
    check(pq!(#[attr(abc = "def")]), Attr(pq!("def")));
}

#[test]
fn test_tuple_requied_named_2() {
    #[derive(StructMeta, PartialEq, Debug)]
    struct Attr(#[name("a")] LitStr, #[name("b")] LitInt);
    check_msg(pq!(#[attr(a = "x", b = 12)]), Attr(pq!("x"), pq!(12)), "ab");
}

#[test]
fn test_tuple_optional_named() {
    #[derive(StructMeta, PartialEq, Debug)]
    struct Attr(#[name("abc")] Option<LitStr>);
    check_msg(pq!(#[attr()]), Attr(None), "args 0");
    check_msg(pq!(#[attr(abc = "def")]), Attr(Some(pq!("def"))), "args 1");
}

#[test]
fn test_struct_value() {
    #[derive(StructMeta, PartialEq, Debug)]
    struct Attr {
        abc: LitStr,
    }
    check(pq!(#[attr(abc = "def")]), Attr { abc: pq!("def") });
}

#[test]
fn test_struct_option_value() {
    #[derive(StructMeta, PartialEq, Debug)]
    struct Attr {
        abc: Option<LitStr>,
    }
    check(pq!(#[attr()]), Attr { abc: None });
    check(
        pq!(#[attr(abc = "def")]),
        Attr {
            abc: Some(pq!("def")),
        },
    );
}

#[test]
fn test_struct_value_raw() {
    #[derive(StructMeta, PartialEq, Debug)]
    struct Attr {
        r#abc: LitStr,
    }
    check(pq!(#[attr(abc = "def")]), Attr { abc: pq!("def") });
}
#[test]
fn test_struct_value_raw_keyword() {
    #[derive(StructMeta, PartialEq, Debug)]
    struct Attr {
        r#if: LitStr,
    }
    check(pq!(#[attr(if = "def")]), Attr { r#if: pq!("def") });
}
#[test]
fn test_struct_value_name() {
    #[derive(StructMeta, PartialEq, Debug)]
    struct Attr {
        #[name("xxx")]
        abc: LitStr,
    }
    check(pq!(#[attr(xxx = "def")]), Attr { abc: pq!("def") });
}

#[test]
fn test_struct_value_name_keyword() {
    #[derive(StructMeta, PartialEq, Debug)]
    struct Attr {
        #[name("if")]
        abc: LitStr,
    }
    check(pq!(#[attr(if = "def")]), Attr { abc: pq!("def") });
}

#[test]
fn test_struct_value_name_self() {
    #[derive(StructMeta, PartialEq, Debug)]
    struct Attr {
        #[name("self")]
        abc: LitStr,
    }
    check(pq!(#[attr(self = "def")]), Attr { abc: pq!("def") });
}

#[test]
fn test_struct_value_unnamed() {
    #[derive(StructMeta, PartialEq, Debug)]
    struct Attr {
        #[unnamed]
        abc: LitStr,
    }
    check(pq!(#[attr("def")]), Attr { abc: pq!("def") });
}

#[test]
fn test_struct_vec() {
    #[derive(StructMeta, PartialEq, Debug)]
    struct Attr {
        abc: Vec<LitStr>,
    }
    check_msg(pq!(#[attr(abc())]), Attr { abc: vec![] }, "args 0");
    check_msg(
        pq!(#[attr(abc("item1"))]),
        Attr {
            abc: vec![pq!("item1")],
        },
        "args 1",
    );
    check_msg(
        pq!(#[attr(abc("item1", "item2"))]),
        Attr {
            abc: vec![pq!("item1"), pq!("item2")],
        },
        "args 2",
    );
}
#[test]
fn test_struct_option_vec() {
    #[derive(StructMeta, PartialEq, Debug)]
    struct Attr {
        abc: Option<Vec<LitStr>>,
    }
    check_msg(pq!(#[attr()]), Attr { abc: None }, "args none");
    check_msg(pq!(#[attr(abc())]), Attr { abc: Some(vec![]) }, "args 0");
    check_msg(
        pq!(#[attr(abc("item1"))]),
        Attr {
            abc: Some(vec![pq!("item1")]),
        },
        "args 1",
    );
    check_msg(
        pq!(#[attr(abc("item1", "item2"))]),
        Attr {
            abc: Some(vec![pq!("item1"), pq!("item2")]),
        },
        "args 2",
    );
}

#[test]
fn test_struct_vec_unnamed() {
    #[derive(StructMeta, PartialEq, Debug)]
    struct Attr {
        #[unnamed]
        abc: Vec<LitStr>,
    }
    check_msg(pq!(#[attr()]), Attr { abc: vec![] }, "args 0");
    check_msg(
        pq!(#[attr("item1")]),
        Attr {
            abc: vec![pq!("item1")],
        },
        "args 1",
    );
    check_msg(
        pq!(#[attr("item1", "item2")]),
        Attr {
            abc: vec![pq!("item1"), pq!("item2")],
        },
        "args 2",
    );
}

#[test]
fn test_struct_flag() {
    #[derive(StructMeta, PartialEq, Debug)]
    struct Attr {
        is_enable: Flag,
    }
    check(
        pq!(#[attr(is_enable)]),
        Attr {
            is_enable: true.into(),
        },
    );
    check(
        pq!(#[attr()]),
        Attr {
            is_enable: false.into(),
        },
    );
}

#[test]
fn test_struct_name_value() {
    #[derive(StructMeta, PartialEq, Debug)]
    struct Attr {
        abc: NameValue<LitStr>,
    }
    check(
        pq!(#[attr(abc = "xyz")]),
        Attr {
            abc: NameValue {
                span: Span::call_site(),
                value: pq!("xyz"),
            },
        },
    );
}
#[test]
fn test_struct_name_args() {
    #[derive(StructMeta, PartialEq, Debug)]
    struct Attr {
        abc: NameArgs<LitStr>,
    }
    check(
        pq!(#[attr(abc("xyz"))]),
        Attr {
            abc: NameArgs {
                span: Span::call_site(),
                args: pq!("xyz"),
            },
        },
    );
}

fn check<T: Parse + PartialEq + Debug>(input: Attribute, expected: T) {
    check_msg(input, expected, "")
}
fn check_msg<T: Parse + PartialEq + Debug>(input: Attribute, expected: T, msg: &str) {
    match input.parse_args::<T>() {
        Ok(value) => {
            assert_eq!(value, expected, "{}", msg);
        }
        Err(e) => {
            panic!("{} : parse faield. \n{}", msg, e)
        }
    }
}
