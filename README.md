# StructMeta

[![Crates.io](https://img.shields.io/crates/v/structmeta.svg)](https://crates.io/crates/structmeta)
[![Docs.rs](https://docs.rs/structmeta/badge.svg)](https://docs.rs/structmeta/)
[![Actions Status](https://github.com/frozenlib/structmeta/workflows/CI/badge.svg)](https://github.com/frozenlib/structmeta/actions)

Parse Rust's attribute arguments by defining a struct.

## Documentation

See [`#[derive(StructMeta)]` documentation](https://docs.rs/structmeta/latest/structmeta/derive.StructMeta.html) for details.

## Install

Add this to your Cargo.toml:

```toml
[dependencies]
structmeta = "0.3.0"
proc-macro2 = "1.0.78"
syn = "2.0.48"
quote = "1.0.35"
```

## Example

```rust
use structmeta::StructMeta;
use syn::{parse_quote, Attribute, LitInt, LitStr};

#[derive(StructMeta, Debug)]
struct MyAttr {
    x: LitInt,
    y: LitStr,
}
let attr: Attribute = parse_quote!(#[my_attr(x = 10, y = "abc")]);
let attr: MyAttr = attr.parse_args().unwrap();
println!("x = {}, y = {}", attr.x, attr.y.value());
```

This code outputs:

```txt
x = 10, y = abc
```

## License

This project is dual licensed under Apache-2.0/MIT. See the two LICENSE-\* files for details.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
