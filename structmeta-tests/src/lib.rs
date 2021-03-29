use proc_macro::TokenStream;
use quote::quote;
use structmeta::{NameArgs, Parse, StructMeta};
use syn::{parse, parse2, parse_macro_input, DeriveInput, LitInt, LitStr};

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

#[proc_macro_attribute]
pub fn my_attr(attr: TokenStream, _item: TokenStream) -> TokenStream {
    let attr = parse::<MyAttr>(attr).unwrap();
    let msg = attr.msg.value();
    quote!(const MSG: &str = #msg;).into()
}

#[derive(Parse)]
enum SingleVariant {
    A(LitInt, LitStr),
}

#[proc_macro]
pub fn parse_single_variant(input: TokenStream) -> TokenStream {
    match parse2::<SingleVariant>(input.into()) {
        Ok(_) => quote!(),
        Err(e) => e.into_compile_error(),
    }
    .into()
}

#[derive(StructMeta)]
struct RequiredUnnamedParam2(LitInt, LitInt);

#[proc_macro_attribute]
pub fn attr_required_unnamed_param2(attr: TokenStream, item: TokenStream) -> TokenStream {
    parse_attr::<RequiredUnnamedParam2>(attr, item)
}

#[allow(dead_code)]
#[derive(StructMeta)]
struct RequiredUnnamedParam2Inner {
    value: NameArgs<RequiredUnnamedParam2>,
    after: LitInt,
}

#[proc_macro_attribute]
pub fn attr_required_unnamed_param2_inner(attr: TokenStream, item: TokenStream) -> TokenStream {
    parse_attr::<RequiredUnnamedParam2Inner>(attr, item)
}

fn parse_attr<T: syn::parse::Parse>(attr: TokenStream, item: TokenStream) -> TokenStream {
    match parse::<T>(attr) {
        Ok(_) => item,
        Err(e) => e.into_compile_error().into(),
    }
}
