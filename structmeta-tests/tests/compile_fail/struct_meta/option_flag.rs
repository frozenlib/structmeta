use structmeta::Flag;

#[derive(structmeta::StructMeta)]
struct Example {
    flag: Option<Flag>,
}

fn main() {}
