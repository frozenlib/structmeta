use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::quote;
use structmeta::{NameArgs, NameValue, Parse, StructMeta};
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
        if attr.path().is_ident("my_msg") {
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
struct RequiredUnnamed2(LitInt, LitInt);

#[proc_macro_attribute]
pub fn attr_required_unnamed2(attr: TokenStream, item: TokenStream) -> TokenStream {
    parse_attr::<RequiredUnnamed2>(attr, item)
}

#[allow(dead_code)]
#[derive(StructMeta)]
struct RequiredUnnamed2Inner {
    value: NameArgs<RequiredUnnamed2>,
    after: LitInt,
}

#[proc_macro_attribute]
pub fn attr_required_unnamed2_inner(attr: TokenStream, item: TokenStream) -> TokenStream {
    parse_attr::<RequiredUnnamed2Inner>(attr, item)
}

#[allow(dead_code)]
#[derive(StructMeta)]
struct OptionalFlag {
    value: bool,
}

#[proc_macro_attribute]
pub fn attr_flag(attr: TokenStream, item: TokenStream) -> TokenStream {
    parse_attr::<OptionalFlag>(attr, item)
}

#[allow(dead_code)]
#[derive(StructMeta)]
struct RequiredNameValue {
    value: NameValue<LitInt>,
}

#[proc_macro_attribute]
pub fn attr_required_name_value(attr: TokenStream, item: TokenStream) -> TokenStream {
    parse_attr::<RequiredNameValue>(attr, item)
}

#[allow(dead_code)]
#[derive(StructMeta)]
struct OptionalNameValue {
    value: Option<NameValue<LitInt>>,
}
#[proc_macro_attribute]
pub fn attr_optional_name_value(attr: TokenStream, item: TokenStream) -> TokenStream {
    parse_attr::<OptionalNameValue>(attr, item)
}

#[allow(dead_code)]
#[derive(StructMeta)]
struct RestNameValue {
    m: HashMap<String, NameValue<LitInt>>,
}
#[proc_macro_attribute]
pub fn attr_rest_name_value(attr: TokenStream, item: TokenStream) -> TokenStream {
    parse_attr::<RestNameValue>(attr, item)
}

#[allow(dead_code)]
#[derive(StructMeta)]
struct RequiredAndRestNameValue {
    value: NameValue<LitInt>,
    m: HashMap<String, NameValue<LitInt>>,
}
#[proc_macro_attribute]
pub fn attr_required_and_rest_name_value(attr: TokenStream, item: TokenStream) -> TokenStream {
    parse_attr::<RequiredAndRestNameValue>(attr, item)
}

#[allow(dead_code)]
#[derive(StructMeta)]
struct RequiredNameArgs {
    value: NameArgs<LitInt>,
}

#[proc_macro_attribute]
pub fn attr_required_name_args(attr: TokenStream, item: TokenStream) -> TokenStream {
    parse_attr::<RequiredNameArgs>(attr, item)
}

#[allow(dead_code)]
#[derive(StructMeta)]
struct OptionalNameArgs {
    value: Option<NameArgs<LitInt>>,
}

#[proc_macro_attribute]
pub fn attr_optional_name_args(attr: TokenStream, item: TokenStream) -> TokenStream {
    parse_attr::<OptionalNameArgs>(attr, item)
}

#[allow(dead_code)]
#[derive(StructMeta)]
struct RequiredNameArgsOrFlag {
    value: NameArgs<Option<LitInt>>,
}

#[proc_macro_attribute]
pub fn attr_required_name_args_or_flag(attr: TokenStream, item: TokenStream) -> TokenStream {
    parse_attr::<RequiredNameArgsOrFlag>(attr, item)
}

fn parse_attr<T: syn::parse::Parse>(attr: TokenStream, item: TokenStream) -> TokenStream {
    match parse::<T>(attr) {
        Ok(_) => item,
        Err(e) => e.into_compile_error().into(),
    }
}
