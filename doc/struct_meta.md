Derive [`syn::parse::Parse`] for parsing attribute arguments.

- [Example](#example)
- [Named parameter](#named-parameter)
  - [Supported field types for named parameter](#supported-field-types-for-named-parameter)
  - [Flag style](#flag-style)
  - [NameValue style](#namevalue-style)
  - [NameArgs style](#nameargs-style)
  - [NameArgs or Flag style](#nameargs-or-flag-style)
  - [NameArgList style](#namearglist-style)
  - [NameArgList or Flag style](#namearglist-or-flag-style)
  - [Optional named parameter](#optional-named-parameter)
  - [Rest named parameter](#rest-named-parameter)
- [Unnamed parameter](#unnamed-parameter)
  - [Required unnamed parameter](#required-unnamed-parameter)
  - [Optional unnamed parameter](#optional-unnamed-parameter)
  - [Variadic unnamed parameter](#variadic-unnamed-parameter)
- [Parameter order](#parameter-order)
- [Helper attribute `#[struct_meta(...)]`](#helper-attribute-struct_meta)
- [Uses with `#[proc_macro_derive]`](#uses-with-proc_macro_derive)
- [Uses with `#[proc_macro_attribute]`](#uses-with-proc_macro_attribute)
- [Parsing ambiguous arguments](#parsing-ambiguous-arguments)
  - [`#[struct_meta(name_filter = "...")]`](#struct_metaname_filter--)

# Example

```rust
use structmeta::StructMeta;
use syn::{parse_quote, Attribute};
use syn::{LitInt, LitStr};

#[derive(StructMeta)]
struct Args {
    #[struct_meta(unnamed)]
    a: LitStr,
    b: LitInt,
    c: Option<LitInt>,
}

let attr: Attribute = parse_quote!(#[attr("xyz", b = 10)]);
let args: Args = attr.parse_args()?;
assert_eq!(args.a.value(), "xyz");
assert_eq!(args.b.base10_parse::<u32>()?, 10);
assert!(args.c.is_none());
# syn::Result::Ok(())
```

# Named parameter

The following field will be "Named parameter".

- field in record struct.
- field with `#[struct_meta(name = "...")]` in tuple struct.
- However, fields that meet the following conditions are excluded
  - field with `#[struct_meta(unnamed)]`
  - field with type `HashMap<String, _>`

"Named parameter" is a parameter that specifies with a name, such as `#[attr(flag, x = 10, y(1, 2, 3))]`.

## Supported field types for named parameter

"Named parameter" has the following four styles, and the style is determined by the type of the field.

- Flag style : `name`
- NameValue style : `name = value`
- NameArgs style : `name(args)`
- NameArgList style : `name(arg, arg, ...)`

| field type | field type (with span)       | style                                             | example                         |
| ---------- | ---------------------------- | ------------------------------------------------- | ------------------------------- |
| `bool`     | [`Flag`]                     | [Flag](#flag-style)                               | `name`                          |
| `T`        | [`NameValue<T>`]             | [NameValue](#namevalue-style)                     | `name = value`                  |
|            | [`NameArgs<T>`]              | [NameArgs](#nameargs-or-flag-style)               | `name(args)`                    |
|            | [`NameArgs<Option<T>>`]      | [NameArgs or Flag](#nameargs-or-flag-style)       | `name(args)` or `name`          |
| `Vec<T>`   | [`NameArgs<Vec<T>>`]         | [NameArgList](#namearglist-style)                 | `name(arg, arg, ...)`           |
|            | [`NameArgs<Option<Vec<T>>>`] | [NameArgList or Flag](#namearglist-or-flag-style) | `name(arg, arg, ...)` or `name` |

Note: the type `T` in the table above needs to implement `syn::parse::Parse`.

With the above type as P (`bool` and `Flag` are excluded), you can also use the following types.

| field type           | effect                                          |
| -------------------- | ----------------------------------------------- |
| `Option<P>`          | [optional parameter](#optional-named-parameter) |
| `HashMap<String, P>` | [rest parameter](#rest-named-parameter)         |

## Flag style

A field with the type `bool` will be a parameter that specifies only its name.

```rust
use structmeta::StructMeta;
use syn::{parse_quote, Attribute};

#[derive(StructMeta)]
struct Args {
    a: bool,
    b: bool,
}

let attr: Attribute = parse_quote!(#[attr(a)]);
let args: Args = attr.parse_args()?;
assert_eq!(args.a, true);
assert_eq!(args.b, false);

let attr: Attribute = parse_quote!(#[attr(a, b)]);
let args: Args = attr.parse_args()?;
assert_eq!(args.a, true);
assert_eq!(args.b, true);
# syn::Result::Ok(())
```

If you use `Flag` instead of `bool`, you will get its `Span` when the argument is specified.

```rust
use structmeta::{Flag, StructMeta};
use syn::{parse_quote, Attribute};

#[derive(StructMeta)]
struct Args {
    a: Flag,
}

let attr: Attribute = parse_quote!(#[attr(a)]);
let args: Args = attr.parse_args()?;
if let Some(_span) = args.a.span {
    // Use span.
}
# syn::Result::Ok(())
```

## NameValue style

A field with type `T` or `NameValue<T>` will be `name = value` style parameter.

```rust
use structmeta::{NameValue, StructMeta};
use syn::{parse_quote, Attribute, LitInt, LitStr};

#[derive(StructMeta)]
struct Args {
    a: LitStr,
    b: NameValue<LitInt>,
}

let attr: Attribute = parse_quote!(#[attr(a = "abc", b = 10)]);
let args: Args = attr.parse_args()?;
assert_eq!(args.a.value(), "abc");
assert_eq!(args.b.value.base10_parse::<u32>()?, 10);
# syn::Result::Ok(())
```

## NameArgs style

A field with type `NameArgs<T>` will be `name(args)` style parameter.

```rust
use structmeta::{NameArgs, StructMeta};
use syn::{parse_quote, Attribute, LitInt, LitStr};

#[derive(StructMeta)]
struct Args {
    a: NameArgs<LitStr>,
    b: NameArgs<LitInt>,
}

let attr: Attribute = parse_quote!(#[attr(a("abc"), b(10))]);
let args: Args = attr.parse_args()?;
assert_eq!(args.a.args.value(), "abc");
assert_eq!(args.b.args.base10_parse::<u32>()?, 10);
# syn::Result::Ok(())
```

## NameArgs or Flag style

A field with type `NameArgs<T>` will be `name(args)` or `name` style parameter.

```rust
use structmeta::{NameArgs, StructMeta};
use syn::{parse_quote, Attribute, LitInt, LitStr};

#[derive(StructMeta)]
struct Args {
    a: NameArgs<Option<LitStr>>,
    b: NameArgs<Option<LitInt>>,
}

let attr: Attribute = parse_quote!(#[attr(a, b(10))]);
let args: Args = attr.parse_args()?;
assert!(args.a.args.is_none());
assert_eq!(args.b.args.unwrap().base10_parse::<u32>()?, 10);
# syn::Result::Ok(())
```

## NameArgList style

A field with type `NameArgs<Vec<T>>` will be `name(arg, arg, ...)` style parameter.

```rust
use structmeta::{NameArgs, StructMeta};
use syn::{parse_quote, Attribute, LitStr};

#[derive(StructMeta)]
struct Args {
    a: NameArgs<Vec<LitStr>>,
}

let attr: Attribute = parse_quote!(#[attr(a())]);
let args: Args = attr.parse_args()?;
assert_eq!(args.a.args.len(), 0);

let attr: Attribute = parse_quote!(#[attr(a("x"))]);
let args: Args = attr.parse_args()?;
assert_eq!(args.a.args.len(), 1);

let attr: Attribute = parse_quote!(#[attr(a("x", "y"))]);
let args: Args = attr.parse_args()?;
assert_eq!(args.a.args.len(), 2);
# syn::Result::Ok(())
```

## NameArgList or Flag style

A field with type `NameArgs<Option<Vec<T>>>` will be `name(arg, arg, ...)` or `name` style parameter.

```rust
use structmeta::{NameArgs, StructMeta};
use syn::{parse_quote, Attribute, LitStr};

#[derive(StructMeta)]
struct Args {
    abc: NameArgs<Option<Vec<LitStr>>>,
}

let attr: Attribute = parse_quote!(#[attr(abc)]);
let args: Args = attr.parse_args()?;
assert_eq!(args.abc.args.is_none(), true);

let attr: Attribute = parse_quote!(#[attr(abc())]);
let args: Args = attr.parse_args()?;
assert_eq!(args.abc.args.unwrap().len(), 0);

let attr: Attribute = parse_quote!(#[attr(abc("x"))]);
let args: Args = attr.parse_args()?;
assert_eq!(args.abc.args.unwrap().len(), 1);

let attr: Attribute = parse_quote!(#[attr(abc("x", "y"))]);
let args: Args = attr.parse_args()?;
assert_eq!(args.abc.args.unwrap().len(), 2);
# syn::Result::Ok(())
```

## Optional named parameter

If you use `Option` for the field type, it becomes an optional parameter.

```rust
use structmeta::{NameValue, StructMeta};
use syn::{parse_quote, Attribute, LitInt, LitStr};

#[derive(StructMeta)]
struct Args {
    a: Option<LitStr>,
    b: Option<NameValue<LitInt>>,
}

let attr: Attribute = parse_quote!(#[attr(a = "abc")]);
let args: Args = attr.parse_args()?;
assert!(args.a.is_some());
assert!(args.b.is_none());

let attr: Attribute = parse_quote!(#[attr(b = 10)]);
let args: Args = attr.parse_args()?;
assert!(args.a.is_none());
assert!(args.b.is_some());
# syn::Result::Ok(())
```

## Rest named parameter

If `HashMap<String, _>` is used for the field type, the field will contain named arguments that are not associated with the field.

```rust
use std::collections::HashMap;
use structmeta::StructMeta;
use syn::{parse_quote, Attribute, LitInt};

#[derive(StructMeta)]
struct Args {
    a: Option<LitInt>,
    rest: HashMap<String, LitInt>,
}

let attr: Attribute = parse_quote!(#[attr(a = 10, b = 20, c = 30)]);
let args: Args = attr.parse_args()?;
assert!(args.a.is_some());
let mut keys: Vec<_> = args.rest.keys().collect();
keys.sort();
assert_eq!(keys, vec!["b", "c"]);
# syn::Result::Ok(())
```

# Unnamed parameter

The following field will be "Unnamed parameter".

- field in tuple struct.
- field with `#[struct_meta(unnamed)]` in record struct.
- However, fields that meet the following conditions are excluded
  - field with `#[struct_meta(name = "...")]`
  - field with type `HashMap<String, _>`

"Unnamed parameter" is a value-only parameter, such as `#[attr("abc", 10, 20)]`.

| field type  | effect                                            |
| ----------- | ------------------------------------------------- |
| `T`         | [required parameter](#required-unnamed-parameter) |
| `Option<T>` | [optional parameter](#optional-unnamed-parameter) |
| `Vec<T>`    | [variadic parameter](#variadic-unnamed-parameter) |

The type `T` in the table above needs to implement `syn::parse::Parse`.

## Required unnamed parameter

Fields of the type that implement `syn::parse::Parse` will be required parameters.

```rust
use structmeta::StructMeta;
use syn::{parse_quote, Attribute, LitStr, Result};

#[derive(StructMeta)]
struct Args(LitStr);

let attr: Attribute = parse_quote!(#[attr()]);
let args: Result<Args> = attr.parse_args();
assert!(args.is_err());

let attr: Attribute = parse_quote!(#[attr("a")]);
let args: Args = attr.parse_args()?;
assert_eq!(args.0.value(), "a");
# syn::Result::Ok(())
```

## Optional unnamed parameter

Fields of type `Option` will be optional parameters.

```rust
use structmeta::StructMeta;
use syn::{parse_quote, Attribute, LitStr};

#[derive(StructMeta)]
struct Args(Option<LitStr>);

let attr: Attribute = parse_quote!(#[attr()]);
let args: Args = attr.parse_args()?;
assert!(args.0.is_none());

let attr: Attribute = parse_quote!(#[attr("a")]);
let args: Args = attr.parse_args()?;
assert_eq!(args.0.unwrap().value(), "a");
# syn::Result::Ok(())
```

## Variadic unnamed parameter

If you use `Vec` as the field type, multiple arguments can be stored in a single field.

```rust
use structmeta::StructMeta;
use syn::{parse_quote, Attribute, LitStr};

#[derive(StructMeta)]
struct Args(Vec<LitStr>);

let attr: Attribute = parse_quote!(#[attr()]);
let args: Args = attr.parse_args()?;
assert_eq!(args.0.len(), 0);

let attr: Attribute = parse_quote!(#[attr("a")]);
let args: Args = attr.parse_args()?;
assert_eq!(args.0.len(), 1);
assert_eq!(args.0[0].value(), "a");

let attr: Attribute = parse_quote!(#[attr("a", "b")]);
let args: Args = attr.parse_args()?;
assert_eq!(args.0.len(), 2);
assert_eq!(args.0[0].value(), "a");
assert_eq!(args.0[1].value(), "b");
# syn::Result::Ok(())
```

# Parameter order

The parameters must be in the following order.

- Unnamed
  - Required
  - Optional
  - Variadic
- Named

# Helper attribute `#[struct_meta(...)]`

| argument                                           | struct | field | effect                                                                                   |
| -------------------------------------------------- | ------ | ----- | ---------------------------------------------------------------------------------------- |
| `dump`                                             | ✔      |       | Causes a compile error and outputs the automatically generated code as an error message. |
| [`name_filter = "..."`](#struct_metaname_filter--) | ✔      |       | Specify how to distinguish between a parameter name and a value of an unnamed parameter. |
| `name = "..."`                                     |        | ✔     | Specify a parameter name.                                                                |
| `unnamed`                                          |        | ✔     | Make the field be treated as an unnamed parameter.                                       |

# Uses with `#[proc_macro_derive]`

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

# Uses with `#[proc_macro_attribute]`

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

# Parsing ambiguous arguments

If one or more `name = value` style parameters are defined, arguments beginning with `name =` will be parsed as `name = value` style,
even if the name is different from what it is defined as.

This specification is intended to prevent `name = value` from being treated as unnamed parameter due to typo.

```rust
use structmeta::StructMeta;
use syn::{parse_quote, Attribute, Expr, LitInt, Result};

#[derive(StructMeta)]
struct WithNamed {
    #[struct_meta(unnamed)]
    unnamed: Option<Expr>,
    x: Option<LitInt>,
}

let attr_x: Attribute = parse_quote!(#[attr(x = 10)]);
let result: WithNamed = attr_x.parse_args().unwrap();
assert_eq!(result.unnamed.is_some(), false);
assert_eq!(result.x.is_some(), true);

// `y = 10` is parsed as a wrong named parameter.
let attr_y: Attribute = parse_quote!(#[attr(y = 10)]);
let result: Result<WithNamed> = attr_y.parse_args();
assert!(result.is_err());
```

If `name = value` style parameter is not defined, it will be parsed as unnamed parameter.

```rust
use structmeta::StructMeta;
use syn::{parse_quote, Attribute, Expr, LitInt, Result};

#[derive(StructMeta)]
struct WithoutNamed {
    #[struct_meta(unnamed)]
    unnamed: Option<Expr>,
}

// `y = 10` is parsed as a unnamed parameter.
let attr_y: Attribute = parse_quote!(#[attr(y = 10)]);
let result: WithoutNamed = attr_y.parse_args().unwrap();
assert_eq!(result.unnamed, Some(parse_quote!(y = 10)));
```

Similarly, if one or more `name(args)` style parameters are defined, arguments with `name(args)` will be parsed as `name(args)` style.
If `name(args)` style parameter is not defined, it will be parsed as unnamed parameter.

The same is true for `name` style parameter.

## `#[struct_meta(name_filter = "...")]`

By attaching `#[struct_meta(name_filter = "...")]` to struct definition, you can restrict the names that can be used as parameter names and treat other identifiers as a value of unnamed parameter.

The following value can be used.

- `"snake_case"`

```rust
use structmeta::StructMeta;
use syn::{parse_quote, Attribute, Expr, LitInt, Result};

let attr: Attribute = parse_quote!(#[attr(X)]);

#[derive(StructMeta)]
struct NoFilter {
    #[struct_meta(unnamed)]
    unnamed: Option<Expr>,
    x: bool,
}
let result: Result<NoFilter> = attr.parse_args();
assert!(result.is_err()); // `X` is parsed as a wrong named parameter.

#[derive(StructMeta)]
#[struct_meta(name_filter = "snake_case")]
struct WithFilter {
    #[struct_meta(unnamed)]
    unnamed: Option<Expr>,
    x: bool,
}
let result: WithFilter = attr.parse_args().unwrap();
assert_eq!(result.unnamed, Some(parse_quote!(X))); // `X` is parsed as a unnamed parameter.
assert_eq!(result.x, false);
```
