/*!
Parse Rust's attribute arguments by defining a struct.

See [`#[derive(StructMeta)]`](macro@StructMeta) documentation for details.
*/

#[doc(hidden)]
pub mod helpers;

use proc_macro2::{Ident, Span};
use std::{fmt::Display, str::FromStr};
use syn::{
    ext::IdentExt,
    parse::{Parse, ParseStream},
    Error, LitFloat, LitInt, LitStr, Result,
};

// #[include_doc = "../../doc/to_tokens.md"]
/**
Derive [`quote::ToTokens`] for syntax tree node.

`#[derive(ToTokens)]` generates an implementation of `ToTokens` that calls [`ToTokens::to_tokens`](quote::ToTokens::to_tokens) for each field.

```rust
use syn::LitInt;

#[derive(structmeta::ToTokens)]
struct Example(LitInt, LitInt);
```

Code like this will be generated:

```rust
# use syn::LitInt;
# struct Example(LitInt, LitInt);
impl quote::ToTokens for Example {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.0.to_tokens(tokens);
        self.1.to_tokens(tokens);
    }
}
```

`#[derive(ToTokens)]` can also be specified for enum.

```rust
use syn::{LitInt, LitStr};

#[derive(structmeta::ToTokens)]
enum Example {
    A(LitInt),
    B(LitStr),
}
```

Code like this will be generated:

```rust
# use syn::{LitInt, LitStr};
# enum Example {
#    A(LitInt),
#    B(LitStr),
# }
impl quote::ToTokens for Example {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::A(l) => l.to_tokens(tokens),
            Self::B(l) => l.to_tokens(tokens),
        }
    }
}
```

## Helper attributes

|                                                                    | struct | enum | varaint | field |
| ------------------------------------------------------------------ | ------ | ---- | ------- | ----- |
| [`#[to_tokens("[")]`, `#[to_tokens("]")]`](#to_tokens-to_tokens)   |        |      |         | ✔     |
| [`#[to_tokens("(")]`, `#[to_tokens(")")]`](#to_tokens-to_tokens-1) |        |      |         | ✔     |
| [`#[to_tokens("{")]`, `#[to_tokens("}")]`](#to_tokens-to_tokens-2) |        |      |         | ✔     |
| [`#[to_tokens(dump)]`](#to_tokensdump)                             | ✔      | ✔    |         |       |

## `#[to_tokens("[")]`, `#[to_tokens("]")]`

By specifying `#[to_tokens("[")]` for a field of type [`struct@syn::token::Bracket`], subsequent tokens will be enclosed in `[]`.

By default, all subsequent fields are enclosed.
To restrict the enclosing fields, specify `#[to_tokens("]")]` for the field after the end of the enclosure.

```rust
use syn::{token, LitInt};

#[derive(structmeta::ToTokens)]
struct Example {
    x: LitInt,
    #[to_tokens("[")]
    bracket_token: token::Bracket,
    y: LitInt,
    #[to_tokens("]")]
    z: LitInt,
}
```

Code like this will be generated:

```rust
# use syn::{token, LitInt};
#
# struct Example {
#    x: LitInt,
#    bracket_token: token::Bracket,
#    y: LitInt,
#    z: LitInt,
# }
impl quote::ToTokens for Example {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.x.to_tokens(tokens);
        token::Bracket::surround(&self.bracket_token, tokens, |tokens| {
            self.y.to_tokens(tokens);
        });
        self.z.to_tokens(tokens);
    }
}
```

## `#[to_tokens("(")]`, `#[to_tokens(")")]`

By specifying `#[to_tokens("(")]` for a field of type [`struct@syn::token::Paren`], subsequent tokens will be enclosed in `()`.

By default, all subsequent fields are enclosed.
To restrict the enclosing fields, specify `#[to_tokens(")")]` for the field after the end of the enclosure.

## `#[to_tokens("{")]`, `#[to_tokens("}")]`

By specifying `#[to_tokens("{")]` for a field of type [`struct@syn::token::Brace`], subsequent tokens will be enclosed in `{}`.

By default, all subsequent fields are enclosed.
To restrict the enclosing fields, specify `#[to_tokens("}")]` for the field after the end of the enclosure.

## `#[to_tokens(dump)]`

Causes a compile error and outputs the code generated by `#[derive(ToTokens)]` as an error message.
*/
// #[include_doc_end = "../../doc/to_tokens.md"]
pub use structmeta_derive::ToTokens;

// #[include_doc = "../../doc/parse.md"]
/**
Derive [`syn::parse::Parse`] for syntax tree node.

`#[derive(Parse)]` generates an implementation of `Parse` that calls [`Parse::parse`](syn::parse::Parse::parse) for each field.

```rust
use syn::{LitInt, LitStr};

#[derive(structmeta::Parse)]
struct Example(LitInt, LitStr);
```

Code like this will be generated:

```rust
# use syn::{LitInt, LitStr};
# struct Example(LitInt, LitStr);
impl syn::parse::Parse for Example {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _0 = input.parse()?;
        let _1 = input.parse()?;
        return Ok(Example(_0, _1));
    }
}
```

`#[derive(Parse)]` can also be specified for enum.

```rust
use syn::{LitInt, LitStr};

#[derive(structmeta::Parse)]
enum Example {
    A(LitInt, LitInt),
    B(LitStr),
}
```

Code like this will be generated:

```rust
# use syn::{LitInt, LitStr};
# enum Example {
#     A(LitInt, LitInt),
#     B(LitStr),
# }
use syn::parse::discouraged::Speculative;
impl syn::parse::Parse for Example {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let fork = input.fork();
        if let Ok(value) = fork.call(|input| Ok(Example::A(input.parse()?, input.parse()?))) {
            input.advance_to(&fork);
            return Ok(value);
        }

        let fork = input.fork();
        if let Ok(value) = fork.call(|input| Ok(Example::B(input.parse()?))) {
            input.advance_to(&fork);
            return Ok(value);
        }

        Err(input.error("parse failed."))
    }
}
```

## Helper attributes

|                                                                    | struct | enum | varaint | field |
| ------------------------------------------------------------------ | ------ | ---- | ------- | ----- |
| [`#[to_tokens("[")]`, `#[to_tokens("]")]`](#to_tokens-to_tokens)   |        |      |         | ✔     |
| [`#[to_tokens("(")]`, `#[to_tokens(")")]`](#to_tokens-to_tokens-1) |        |      |         | ✔     |
| [`#[to_tokens("{")]`, `#[to_tokens("}")]`](#to_tokens-to_tokens-2) |        |      |         | ✔     |
| [`#[parse(peek)]`](#parsepeek)                                     |        |      |         | ✔     |
| [`#[parse(any)]`](#parseany)                                       |        |      |         | ✔     |
| [`#[parse(terminated)]`](#parseterminated)                         |        |      |         | ✔     |
| [`#[parse(dump)]`](#parsedump)                                     | ✔      | ✔    |         |       |

## `#[to_tokens("[")]`, `#[to_tokens("]")]`

By specifying `#[to_tokens("[")]` for a field of type [`struct@syn::token::Bracket`], subsequent tokens will be enclosed in `[]`.

By default, all subsequent fields are enclosed.
To restrict the enclosing fields, specify `#[to_tokens("]")]` for the field after the end of the enclosure.

```rust
use syn::{token, LitInt};

#[derive(structmeta::Parse)]
struct Example {
    x: LitInt,
    #[to_tokens("[")]
    bracket_token: token::Bracket,
    y: LitInt,
    #[to_tokens("]")]
    z: LitInt,
}
```

Code like this will be generated:

```rust
# use syn::{token, LitInt};
#
# struct Example {
#    x: LitInt,
#    bracket_token: token::Bracket,
#    y: LitInt,
#    z: LitInt,
# }
impl syn::parse::Parse for Example {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let x = input.parse()?;
        let content;
        let bracket_token = syn::bracketed!(content in input);
        let y = content.parse()?;
        let z = input.parse()?;
        Ok(Self {
            x,
            bracket_token,
            y,
            z,
        })
    }
}
```

## `#[to_tokens("(")]`, `#[to_tokens(")")]`

By specifying `#[to_tokens("(")]` for a field of type [`struct@syn::token::Paren`], subsequent tokens will be enclosed in `()`.

By default, all subsequent fields are enclosed.
To restrict the enclosing fields, specify `#[to_tokens(")")]` for the field after the end of the enclosure.

## `#[to_tokens("{")]`, `#[to_tokens("}")]`

By specifying `#[to_tokens("{")]` for a field of type [`struct@syn::token::Brace`], subsequent tokens will be enclosed in `{}`.

By default, all subsequent fields are enclosed.
To restrict the enclosing fields, specify `#[to_tokens("}")]` for the field after the end of the enclosure.

## `#[parse(peek)]`

When parsing an enum, it will peek the field with this attribute set,
and if successful, will parse the variant containing the field.
If the peek succeeds, the subsequent variant will not be parsed even if the parse failed.

Variant where `#[parse(peek)]` is not specified will fork input and parse.

If the peek fails or the parsing of the forked input fails, the subsequent variant will be parsed.

```rust
use syn::{LitInt, LitStr};
#[derive(structmeta::Parse)]
enum Example {
    A(#[parse(peek)] LitInt, LitInt),
    B(LitStr),
}
```

Code like this will be generated:

```rust
# use syn::{LitInt, LitStr};
# enum Example {
#     A(LitInt, LitInt),
#     B(LitStr),
# }
impl syn::parse::Parse for Example {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(LitInt) {
            let a_0 = input.parse()?;
            let a_1 = input.parse()?;
            return Ok(Example::A(a_0, a_1));
        }
        let b_0 = input.parse()?;
        Ok(Example::B(b_0))
    }
}
```

`#[parse(peek)]` can specified on first three token tree for each variants.

```rust
use syn::{LitInt, LitStr};
#[derive(structmeta::Parse)]
enum Example {
    A(#[parse(peek)] LitInt, #[parse(peek)]LitInt, #[parse(peek)]LitInt),
    B(#[parse(peek)] LitStr),
}
```

Since the tokens enclosed by the delimiter is treated as a single token tree, you can also specify `#[parse(peek)]` to the field with `#[to_tokens("]")]`, `#[to_tokens("}")]`, `#[to_tokens(")")]`.

```rust
use syn::{token, LitInt, LitStr};
#[derive(structmeta::Parse)]
enum Example {
    A {
        #[parse(peek)]
        #[to_tokens("{")]
        a: token::Brace,
        b: LitInt,
        c: LitInt,
        #[to_tokens("}")]
        #[parse(peek)]
        d: LitInt,
    },
}
```

To use `#[parse(peek)]` for a field that type is `Ident`, use `syn::Ident` insted of `proc_macro2::Ident`.

```compile_fail
#[derive(structmeta::Parse)]
enum ExampleNg {
    A(#[parse(peek)] proc_macro2::Ident),
}
```

```rust
#[derive(structmeta::Parse)]
enum ExampleOk {
    A(#[parse(peek)] syn::Ident),
}
```

## `#[parse(any)]`

When parsing `Ident`, allow values that cannot be used as identifiers, such as keywords.

In other words, `Ident::parse_any` and `Ident::peek_any` was generated instead of `Ident::parse` and `Ident::peek`.

```rust
use quote::quote;
use structmeta::Parse;
use syn::{parse2, Ident};

#[derive(Parse)]
struct WithAny(#[parse(any)] Ident);

#[derive(Parse)]
struct WithoutAny(Ident);

assert_eq!(parse2::<WithAny>(quote!(self)).is_ok(), true);
assert_eq!(parse2::<WithoutAny>(quote!(self)).is_ok(), false);
```

## `#[parse(terminated)]`

Use [`Punctuated::parse_terminated`](syn::punctuated::Punctuated::parse_terminated) to parse.

```rust
use quote::quote;
use structmeta::Parse;
use syn::{parse2, punctuated::Punctuated, Ident, Token};
#[derive(Parse)]
struct Example(#[parse(terminated)] Punctuated<Ident, Token![,]>);
assert_eq!(parse2::<Example>(quote!(a, b, c)).is_ok(), true);
```

`terminated` can also be used with `any`.

```rust
use quote::quote;
use structmeta::Parse;
use syn::{parse2, punctuated::Punctuated, Ident, Token};

#[derive(Parse)]
struct WithAny(#[parse(terminated, any)] Punctuated<Ident, Token![,]>);

#[derive(Parse)]
struct WithoutAny(#[parse(terminated)] Punctuated<Ident, Token![,]>);

assert_eq!(parse2::<WithAny>(quote!(self, self)).is_ok(), true);
assert_eq!(parse2::<WithoutAny>(quote!(self, self)).is_ok(), false);
```

## `#[parse(dump)]`

Causes a compile error and outputs the code generated by `#[derive(Parse)]` as an error message.
*/
// #[include_doc_end = "../../doc/parse.md"]
pub use structmeta_derive::Parse;

// #[include_doc = "../../doc/struct_meta.md"]
/**
Derive [`syn::parse::Parse`] for parsing attribute arguments.

## Uses with `#[proc_macro_derive]`

A type with `#[derive(StructMeta)]` can be used with [`syn::Attribute::parse_args`].

```rust
# extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use structmeta::StructMeta;
use syn::{parse, parse_macro_input, DeriveInput, LitStr};

#[derive(StructMeta)]
struct MyAttr {
    msg: LitStr,
}

# const IGNORE_TOKENS: &str = stringify! {
#[proc_macro_derive(MyMsg, attributes(my_msg))]
# };
pub fn derive_my_msg(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let mut msg = String::new();
    for attr in input.attrs {
        if attr.path.is_ident("my_msg") {
            let attr = attr.parse_args::<MyAttr>().unwrap();
            msg = attr.msg.value();
        }
    }
    quote!(const MSG: &str = #msg;).into()
}
```

```ignore
#[derive(MyMsg)]
#[my_msg(msg = "abc")]
struct TestType;

assert_eq!(MSG, "abc");
```

## Uses with `#[proc_macro_attribute]`

A type with `#[derive(StructMeta)]` can be used with `attr` parameter in attribute proc macro.

```rust
# extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use structmeta::StructMeta;
use syn::{parse, parse_macro_input, DeriveInput, LitStr};

#[derive(StructMeta)]
struct MyAttr {
    msg: LitStr,
}
# const IGNORE_TOKENS: &str = stringify! {
#[proc_macro_attribute]
# };
pub fn my_attr(attr: TokenStream, _item: TokenStream) -> TokenStream {
    let attr = parse::<MyAttr>(attr).unwrap();
    let msg = attr.msg.value();
    quote!(const MSG: &str = #msg;).into()
}
```

```ignore
#[my_attr(msg = "xyz")]
struct TestType;

assert_eq!(MSG, "xyz");
```

## Named parameter

The following field will be "Named parameter".

- field in record struct.
- field with `#[struct_meta(name = "...")]` in tuple struct.

"Named parameter" is a parameter that specifies the name, such as `#[attr(x = "abc", y = 10, z = 20)]`.

"Named parameter" has the following four styles, and the style is determined by the type of the field.

- Flag style : `name`
- NameValue style : `name = value`
- NameArgs style : `name(args)`
- NameArgList style : `name(arg, arg, ...)`

| field type (without span) | field type (with span)       | style                           | can be use with `Option` |
| ------------------------- | ---------------------------- | ------------------------------- | ------------------------ |
| `bool`                    | [`Flag`]                     | `name`                          |                          |
| `T`                       | [`NameValue<T>`]             | `name = value`                  | ✔                        |
|                           | [`NameArgs<T>`]              | `name(args)`                    | ✔                        |
|                           | [`NameArgs<Option<T>>`]      | `name(args)` or `name`          | ✔                        |
| `Vec<T>`                  | [`NameArgs<Vec<T>>`]         | `name(arg, arg, ...)`           | ✔                        |
|                           | [`NameArgs<Option<Vec<T>>>`] | `name(arg, arg, ...)` or `name` | ✔                        |

The type `T` in the table above needs to implement `syn::parse::Parse`.

The type in `field type (with span)` column of the table above holds `Span` of parameter name.

Some types can be wrap with `Option` to make them optional parameters.

Some types can be wrap with `HashMap<String, _>` to make them arbitrary name parameters.

## Unnamed parameter

The following field will be "Unnamed parameter".

- field in tuple struct.
- field with `#[struct_meta(unnamed)]` in record struct.

"Unnamed parameter" is a value-only parameter, such as `#[attr("abc", 10, 20)]`.

| field type  | effect             |
| ----------- | ------------------ |
| `T`         | required parameter |
| `Option<T>` | optional parameter |
| `Vec<T>`    | variadic parameter |

The type `T` in the table above needs to implement `syn::parse::Parse`.

## Parameter order

The parameters must be in the following order.

- Unnamed
  - Required
  - Optional
  - Variadic
- Named

## Helper attribute `#[struct_meta(...)]`

### Struct attribute arguments

| argument | effect                                                                                   |
| -------- | ---------------------------------------------------------------------------------------- |
| dump     | Causes a compile error and outputs the automatically generated code as an error message. |

### Field attribute arguments

| argument     | effect                                             |
| ------------ | -------------------------------------------------- |
| name = "..." | Specify a parameter name.                          |
| unnamed      | Make the field be treated as an unnamed parameter. |
*/
// #[include_doc_end = "../../doc/struct_meta.md"]
pub use structmeta_derive::StructMeta;

/// `name` style attribute argument.
#[derive(Clone, Debug, Default)]
pub struct Flag {
    pub span: Option<Span>,
}
impl Flag {
    pub const NONE: Flag = Flag { span: None };
    pub fn value(&self) -> bool {
        self.span.is_some()
    }
}
impl PartialEq for Flag {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}
impl From<bool> for Flag {
    fn from(value: bool) -> Self {
        Self {
            span: if value { Some(Span::call_site()) } else { None },
        }
    }
}

/// `name = value` style attribute argument.
#[derive(Copy, Clone, Debug)]
pub struct NameValue<T> {
    pub name_span: Span,
    pub value: T,
}
impl<T: PartialEq> PartialEq for NameValue<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

/// `name(value)` style attribute argument.
#[derive(Copy, Clone, Debug)]
pub struct NameArgs<T> {
    pub name_span: Span,
    pub args: T,
}
impl<T: PartialEq> PartialEq for NameArgs<T> {
    fn eq(&self, other: &Self) -> bool {
        self.args == other.args
    }
}

pub struct Keyword<T> {
    pub span: Span,
    pub value: T,
}
impl<T: FromStr> Parse for Keyword<T>
where
    T::Err: Display,
{
    fn parse(input: ParseStream) -> Result<Self> {
        let name = Ident::parse_any(input)?;
        let span = name.span();
        match name.to_string().parse() {
            Ok(value) => Ok(Keyword { span, value }),
            Err(e) => Err(Error::new(span, e)),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct LitValue<T> {
    span: Span,
    value: T,
}
impl<T: PartialEq> PartialEq for LitValue<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
impl<T: PartialEq> PartialEq<T> for LitValue<T> {
    fn eq(&self, other: &T) -> bool {
        self.value.eq(other)
    }
}

macro_rules! impl_parse_lit_value_int {
    ($($ty:ty),*) => {
        $(
        impl Parse for LitValue<$ty> {
            fn parse(input: ParseStream) -> Result<Self> {
                let lit = input.parse::<LitInt>()?;
                Ok(Self {
                    span: lit.span(),
                    value: lit.base10_parse()?,
                })
            }
        }
        )*
    };
}

impl_parse_lit_value_int!(u8, u16, u32, u64, u128);
impl_parse_lit_value_int!(i8, i16, i32, i64, i128);

impl Parse for LitValue<String> {
    fn parse(input: ParseStream) -> Result<Self> {
        let lit = input.parse::<LitStr>()?;
        Ok(LitValue {
            span: lit.span(),
            value: lit.value(),
        })
    }
}

impl Parse for LitValue<f32> {
    fn parse(input: ParseStream) -> Result<Self> {
        let lit = input.parse::<LitFloat>()?;
        Ok(LitValue {
            span: lit.span(),
            value: lit.base10_parse()?,
        })
    }
}
impl Parse for LitValue<f64> {
    fn parse(input: ParseStream) -> Result<Self> {
        let lit = input.parse::<LitFloat>()?;
        Ok(LitValue {
            span: lit.span(),
            value: lit.base10_parse()?,
        })
    }
}
