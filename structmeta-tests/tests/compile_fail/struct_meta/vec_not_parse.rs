#[derive(structmeta::StructMeta)]
struct Example {
    x: Vec<NotParse>,
}
struct NotParse;

fn main() {}
