extern crate proc_macro;

#[macro_use]
mod syn_utils;
mod to_tokens;

use syn::{parse_macro_input, DeriveInput};
use syn_utils::*;

#[proc_macro_derive(ToTokens, attributes(to_tokens))]
pub fn derive_to_tokens(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    into_macro_output(to_tokens::derive_to_tokens(input))
}

#[proc_macro_derive(Parse, attributes(parse))]
pub fn derive_parse(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // let input = parse_macro_input!(input as DeriveInput);
    //into_macro_output(arbitrary::derive_arbitrary(input))
    todo!()
}

#[proc_macro_derive(StructMeta, attributes(skip, name))]
pub fn derive_struct_meta(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // let input = parse_macro_input!(input as DeriveInput);
    // into_macro_output(arbitrary::derive_arbitrary(input))
    todo!()
}
