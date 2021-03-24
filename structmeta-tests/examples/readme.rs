use structmeta::StructMeta;
use syn::{parse_quote, Attribute, LitInt, LitStr};

fn main() {
    #[derive(StructMeta, Debug)]
    struct MyAttr {
        x: LitInt,
        y: LitStr,
    }
    let attr: Attribute = parse_quote!(#[my_attr(x = 10, y = "abc")]);
    let attr: MyAttr = attr.parse_args().unwrap();
    println!("x = {}, y = {}", attr.x, attr.y.value());
}
