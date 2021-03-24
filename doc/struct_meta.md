# derive macro `StructMeta`

Derive [`syn::parse::Parse`] for parsing attribute arguments.

## Uses with `#[proc_macro_derive]`

A type with `#[derive(StructMeta)]` can be used with [`syn::Attribute::parse_args`].

```rust
use proc_macro::TokenStream;
use quote::quote;
use structmeta::StructMeta;
use syn::{parse, parse_macro_input, DeriveInput, LitStr};

#[derive(StructMeta)]
struct MyAttr {
    msg: LitStr,
}

#[proc_macro_derive(MyMsg, attributes(my_msg))]
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

```rust
#[derive(MyMsg)]
#[my_msg(msg = "abc")]
struct TestType;

assert_eq!(MSG, "abc");
```

## Uses with `#[proc_macro_attribute]`

A type with `#[derive(StructMeta)]` can be used with `attr` parameter in attribute proc macro.

```rust
use proc_macro::TokenStream;
use quote::quote;
use structmeta::StructMeta;
use syn::{parse, parse_macro_input, DeriveInput, LitStr};

#[derive(StructMeta)]
struct MyAttr {
    msg: LitStr,
}
#[proc_macro_attribute]
pub fn my_attr(attr: TokenStream, _item: TokenStream) -> TokenStream {
    let attr = parse::<MyAttr>(attr).unwrap();
    let msg = attr.msg.value();
    quote!(const MSG: &str = #msg;).into()
}
```

```rust
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
| `bool` (TODO)               | `Flag`                     | `name`                              |                          |
| `T`                         | `NameValue<T>`             | `name = value`                      | ✔                        |
|                             | `NameValue<Option<T>>`     | `name = value` or `name`            | ✔                        |
|                             | `NameArgs<T>`              | `name(args)`                        | ✔                        |
|                             | `NameArgs<Option<T>>`      | `name(args)` or `name`              | ✔                        |
| `Vec<T>`                    | `NameArgs<Vec<T>>`         | `name(arg, arg, ...)`               | ✔                        |
|                             | `NameArgs<Option<Vec<T>>>` | `name(arg, arg, ...)` or `name`     | ✔                        |
| `HashMap<String, T>` (TODO) | `HashMap<Ident, T>` (TODO) | `name1 = value, name2 = value, ...` |                          |

The type `T` in the table above needs to implement `syn::parse::Parse`.

The type in `field type (with span)` column of the table above has `span` member and holds `Span` of parameter name.

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
