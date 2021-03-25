use structmeta::ToTokens;
#[derive(ToTokens)]
struct TestType {
    x: X,
}

struct X;

fn main() {}
