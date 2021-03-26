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
        let value_0 = input.parse()?;
        let value_1 = input.parse()?;
        return Ok(Example(value_0, value_1));
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
# #[derive(structmeta::Parse)]
# enum Example {
#     A(#[parse(peek)] LitInt, LitInt),
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
use syn::{LitInt, LitStr};
#[derive(structmeta::Parse)]
enum Example {
    A(#[to_tokens("{")] LitInt, #[parse(peek)]LitInt, #[parse(peek)]LitInt),
    B(#[parse(peek)] LitStr),
}

```

To use `#[parse(peek)]` for a field that type is `Ident`, use `syn::Ident` insted of `proc_macro2::Ident`.

## `#[parse(any)]`

When parsing `Ident`, allow values that cannot be used as identifiers, such as keywords.

In other words, instead of `Ident::parse` and `Ident::peek`, use `Ident::parse_any` and `Ident::peek_any`.

## `#[parse(terminated)]`

Use `parse_terminated` to parse.

## `#[parse(dump)]`

Causes a compile error and outputs the code generated by `#[derive(Parse)]` as an error message.
