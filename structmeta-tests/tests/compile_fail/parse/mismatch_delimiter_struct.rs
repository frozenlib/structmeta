use structmeta::Parse;
use syn::LitStr;
#[derive(Parse)]
struct TestType {
    #[to_tokens("(")]
    bracket_token: syn::token::Bracket,
    str: LitStr,
}

fn main() {}
