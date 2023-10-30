Derive [`syn::parse::Parse`] for syntax tree node.

- [Example](#example)
- [Helper attributes](#helper-attributes)
  - [`#[to_tokens("[", "]", "(", ")", "{", "}")]`](#to_tokens-----)
  - [`#[parse(peek)]`](#parsepeek)
  - [`#[parse(any)]`](#parseany)
  - [`#[parse(terminated)]`](#parseterminated)
  - [`#[parse(dump)]`](#parsedump)

# Example

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

# Helper attributes

|                                                                 | struct | enum | variant | field |
| --------------------------------------------------------------- | ------ | ---- | ------- | ----- |
| [`#[to_tokens("[", "]", "(", ")", "{", "}")]`](#to_tokens-----) |        |      |         | ✔     |
| [`#[parse(peek)]`](#parsepeek)                                  |        |      |         | ✔     |
| [`#[parse(any)]`](#parseany)                                    |        |      |         | ✔     |
| [`#[parse(terminated)]`](#parseterminated)                      |        |      |         | ✔     |
| [`#[parse(dump)]`](#parsedump)                                  | ✔      | ✔    |         |       |

## `#[to_tokens("[", "]", "(", ")", "{", "}")]`

By specifying `#[to_tokens("[")]` or `#[to_tokens("(")]` or `#[to_tokens("[")]` , subsequent tokens will be enclosed in `[]` or `()` or `{}`.

By default, all subsequent fields are enclosed.
To restrict the enclosing fields, specify `#[to_tokens("]")]` or `#[to_tokens(")")]` or `#[to_tokens("}")]` for the field after the end of the enclosure.

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

If the field type is `Bracket` or `Paren` or `Brace`, the symbol corresponding to the token type must be specified.

If the field type is `MacroDelimiter`, any symbol can be used and there is no difference in behavior. (Three types of parentheses are available, no matter which symbol is specified.)

| field type                     | start                   | end                     |
| ------------------------------ | ----------------------- | ----------------------- |
| [`struct@syn::token::Bracket`] | `"["`                   | `"]"`                   |
| [`struct@syn::token::Paren`]   | `"("`                   | `")"`                   |
| [`struct@syn::token::Brace`]   | `"{"`                   | `"}"`                   |
| [`enum@syn::MacroDelimiter`]   | `"["` or `"("` or `"{"` | `"]"` or `")"` or `"}"` |

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

`#[parse(peek)]` can be specified on the first three `TokenTree` for each variant.

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

To use `#[parse(peek)]` for a field that type is `Ident`, use `syn::Ident` instead of `proc_macro2::Ident`.

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
