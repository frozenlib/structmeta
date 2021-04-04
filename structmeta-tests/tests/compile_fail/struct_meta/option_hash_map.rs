use std::collections::HashMap;
use syn::LitInt;

#[derive(structmeta::StructMeta)]
struct Example {
    rest: Option<HashMap<String, LitInt>>,
}

fn main() {}
