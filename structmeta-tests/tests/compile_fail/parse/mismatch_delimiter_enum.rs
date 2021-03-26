use structmeta::Parse;
use syn::LitStr;
#[derive(Parse)]
enum TestType {
    A {
        #[to_tokens("(")]
        braket_token: syn::token::Bracket,
        str: LitStr,
    },
    B(LitStr),
}

fn main() {}
