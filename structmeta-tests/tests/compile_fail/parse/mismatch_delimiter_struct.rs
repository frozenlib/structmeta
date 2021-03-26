use structmeta::Parse;
use syn::LitStr;
#[derive(Parse)]
struct TestType {
    #[to_tokens("(")]
    braket_token: syn::token::Bracket,
    str: LitStr,
}

fn main() {}
