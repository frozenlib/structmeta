use structmeta::Parse;
use syn::LitStr;
#[derive(Parse)]
struct TestType(#[to_tokens("(")] syn::token::Bracket, LitStr);

fn main() {}
