use std::collections::HashMap;
use structmeta::Flag;

#[derive(structmeta::StructMeta)]
struct Example {
    rest: HashMap<String, Flag>,
}

fn main() {}
