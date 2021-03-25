use structmeta::ToTokens;
use syn::LitStr;
#[derive(ToTokens)]
struct TestType {
    #[to_tokens(xxx = 123)]
    braket_token: syn::token::Bracket,
    str: LitStr,
}

fn main() {}
