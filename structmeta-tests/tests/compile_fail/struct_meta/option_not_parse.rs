#[derive(structmeta::StructMeta)]
struct Example {
    not_parse: Option<NotParse>,
}
struct NotParse;

fn main() {}
