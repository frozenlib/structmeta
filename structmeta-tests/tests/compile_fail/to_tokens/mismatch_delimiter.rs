use structmeta::ToTokens;
use syn::LitStr;
#[derive(ToTokens)]
struct TestType {
    #[to_tokens("(")]
    bracket_token: syn::token::Bracket,
    str: LitStr,
}

fn main() {}
