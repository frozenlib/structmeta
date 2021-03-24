#[doc(hidden)]
pub mod helpers;

use proc_macro2::{Ident, Span};
use std::{fmt::Display, str::FromStr};
use syn::{
    ext::IdentExt,
    parse::{Parse, ParseStream},
    Error, Result,
};

pub use structmeta_derive::{Parse, ToTokens};

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

| field type (without span)   | field type (with span)     | style                               | can be use with `Option` |
| --------------------------- | -------------------------- | ----------------------------------- | ------------------------ |
| `bool`                      | `Flag`                     | `name`                              |                          |
| `T`                         | `NameValue<T>`             | `name = value`                      | ✔                        |
|                             | `NameValue<Option<T>>`     | `name = value` or `name`            | ✔                        |
|                             | `NameArgs<T>`              | `name(args)`                        | ✔                        |
|                             | `NameArgs<Option<T>>`      | `name(args)` or `name`              | ✔                        |
| `Vec<T>`                    | `NameArgs<Vec<T>>`         | `name(arg, arg, ...)`               | ✔                        |
|                             | `NameArgs<Option<Vec<T>>>` | `name(arg, arg, ...)` or `name`     | ✔                        |
| `HashMap<String, T>` (TODO) | `HashMap<Ident, T>` (TODO) | `name1 = value, name2 = value, ...` |                          |

The type `T` in the table above needs to implement `syn::parse::Parse`.

The type in `field type (with span)` column of the table above has `span` member and holds `Span` of parameter name.

Some types can be wrap with `Option` to make them optional parameters.

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

### For struct

| argument | effect                                                                                   |
| -------- | ---------------------------------------------------------------------------------------- |
| dump     | Causes a compile error and outputs the automatically generated code as an error message. |

### For field

| argument     | effect                                             |
| ------------ | -------------------------------------------------- |
| name = "..." | Specify a parameter name.                          |
| unnamed      | Make the field be treated as an unnamed parameter. |
*/
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
#[derive(Debug)]
pub struct NameValue<T> {
    pub span: Span,
    pub value: T,
}
impl<T: PartialEq> PartialEq for NameValue<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

/// `name(value)` style attribute argument.
#[derive(Debug)]
pub struct NameArgs<T> {
    pub span: Span,
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
