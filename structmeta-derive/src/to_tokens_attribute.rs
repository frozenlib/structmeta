use proc_macro2::Span;
use syn::{parse::Parse, spanned::Spanned, LitStr, Result, Token};

pub struct ToTokensAttribute {
    pub dump: Option<Span>,
    pub token: Vec<LitStr>,
}
impl Parse for ToTokensAttribute {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let args = input.parse_terminated(ToTokensAttributeArg::parse, Token![,])?;
        let mut token = Vec::new();
        let mut dump = None;
        for arg in args.into_iter() {
            match arg {
                ToTokensAttributeArg::Token(token_value) => {
                    token.push(token_value);
                }
                ToTokensAttributeArg::Dump(kw_dump) => {
                    if dump.is_none() {
                        dump = Some(kw_dump.span());
                    }
                }
            }
        }
        Ok(Self { dump, token })
    }
}

mod kw {
    use syn::custom_keyword;
    custom_keyword!(dump);
}

enum ToTokensAttributeArg {
    Token(LitStr),
    Dump(kw::dump),
}
impl Parse for ToTokensAttributeArg {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        if input.peek(LitStr) {
            Ok(Self::Token(input.parse()?))
        } else if input.peek(kw::dump) {
            Ok(Self::Dump(input.parse()?))
        } else {
            Err(input.error("expected string literal."))
        }
    }
}

pub fn to_close(c: char) -> char {
    match c {
        '(' => ')',
        '[' => ']',
        '{' => '}',
        _ => panic!("not found closing delimiter for {c}"),
    }
}
