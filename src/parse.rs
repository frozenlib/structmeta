use std::unreachable;

use crate::{syn_utils::*, to_tokens_attribute::*};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse2, parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    token::Paren,
    Data, DataEnum, DataStruct, DeriveInput, Fields, Ident, LitStr, Result, Token,
};

pub fn derive_parse(input: DeriveInput) -> Result<TokenStream> {
    let mut dump = false;
    for attr in &input.attrs {
        if attr.path.is_ident("parse") {
            let attr: ParseAttribute = parse2(attr.tokens.clone())?;
            dump = dump || attr.dump.is_some();
        }
    }

    let ts = match &input.data {
        Data::Struct(data) => code_from_struct(&data)?,
        Data::Enum(data) => code_from_enum(&input.ident, &data)?,
        Data::Union(_) => {
            bail!("Not supported for union.")
        }
    };
    let ts = quote! {
        fn parse(input: ::syn::parse::ParseStream<'_>) -> ::syn::Result<Self> {
            #ts
        }
    };
    let ts = impl_trait_result(&input, &parse_quote!(::syn::parse::Parse), &[], ts, dump)?;
    Ok(ts)
}

fn code_from_struct(data: &DataStruct) -> Result<TokenStream> {
    code_from_fields(quote!(Self), &data.fields)
}
fn code_from_enum(self_ident: &Ident, data: &DataEnum) -> Result<TokenStream> {
    let mut ts = TokenStream::new();
    for variant in &data.variants {
        let variant_ident = &variant.ident;
        let fn_ident = format_ident!("_parse_{}", &variant.ident);
        let fn_expr = code_from_fields(quote!(#self_ident::#variant_ident), &variant.fields)?;
        ts.extend(quote! {
            #[allow(non_snake_case)]
            fn #fn_ident(input: ::syn::parse::ParseStream<'_>) -> ::syn::Result<#self_ident> {
                #fn_expr
            }
            let fork = input.fork();
            if let Ok(value) = #fn_ident(&fork) {
                ::syn::parse::discouraged::Speculative::advance_to(input, &fork);
                return Ok(value);
            }
        });
    }
    ts.extend(quote! {
        Err(input.error("parse failed."))
    });
    Ok(ts)
}
struct Scope {
    input: Ident,
    close: Option<char>,
}
fn code_from_fields(self_path: TokenStream, fields: &Fields) -> Result<TokenStream> {
    let mut scopes = vec![Scope {
        input: parse_quote!(input),
        close: None,
    }];
    let mut ts = TokenStream::new();
    let mut inits = Vec::new();
    for (index, field) in fields.iter().enumerate() {
        let var_ident = to_var_ident(&field.ident, Some(index));
        let mut use_parse = true;
        for attr in &field.attrs {
            if attr.path.is_ident("to_tokens") {
                let attr: ToTokensAttribute = parse2(attr.tokens.clone())?;
                for token in attr.token {
                    for c in token.value().chars() {
                        match c {
                            '(' | '[' | '{' => {
                                use_parse = false;
                                let macro_ident: Ident = match c {
                                    '(' => parse_quote!(parenthesized),
                                    '[' => parse_quote!(bracketed),
                                    '{' => parse_quote!(braced),
                                    _ => unreachable!(),
                                };
                                let input_old = &scopes.last().unwrap().input;
                                let input = format_ident!("input{}", var_ident);
                                let code = quote_spanned!(field.span()=>
                                    let #input;
                                    let #var_ident = ::syn::#macro_ident!(#input in #input_old);
                                    let #input = &#input;
                                );
                                ts.extend(code);
                                scopes.push(Scope {
                                    close: Some(to_close(c)),
                                    input,
                                });
                            }
                            ')' | ']' | '}' => {
                                if scopes.last().unwrap().close == Some(c) {
                                    scopes.pop();
                                } else {
                                    bail!(token.span(), "mismatched closing delimiter `{}`.", c);
                                }
                            }
                            _ => {
                                bail!(
                                    token.span(),
                                    "expected '(', ')', '[', ']', '{{' or '}}', found `{}`.",
                                    c
                                );
                            }
                        }
                    }
                }
            }
        }
        if use_parse {
            let input = &scopes.last().unwrap().input;
            let code =
                quote_spanned!(field.span()=>let #var_ident = ::syn::parse::Parse::parse(#input)?;);
            ts.extend(code);
        }
        if let Some(field_ident) = &field.ident {
            inits.push(quote!(#field_ident : #var_ident));
        } else {
            inits.push(quote!(#var_ident));
        }
    }
    let init = match &fields {
        Fields::Named(_) => quote!({#(#inits,)*}),
        Fields::Unnamed(_) => quote!((#(#inits,)*)),
        Fields::Unit => quote!(),
    };
    Ok(quote! {
        #ts
        Ok(#self_path #init)
    })
}

fn to_var_ident(ident: &Option<Ident>, index: Option<usize>) -> Ident {
    if let Some(ident) = ident {
        format_ident!("_{}", ident)
    } else {
        format_ident!("_{}", index.unwrap())
    }
}

struct ParseAttribute {
    dump: Option<Span>,
    peek: Option<PeekArg>,
}
impl Parse for ParseAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        parenthesized!(content in input);
        let mut peek = None;
        let mut dump = None;
        let args: Punctuated<ParseAttributeArg, Token![,]> =
            content.parse_terminated(ParseAttributeArg::parse)?;
        for arg in args.into_iter() {
            match arg {
                ParseAttributeArg::Peek(peek_arg) => {
                    if peek.is_some() {
                        bail!(peek_arg.span(), "peek(..) is already specified.");
                    }
                    peek = Some(peek_arg);
                }
                ParseAttributeArg::Dump(kw_dump) => {
                    if dump.is_none() {
                        dump = Some(kw_dump.span());
                    }
                }
            }
        }
        Ok(Self { peek, dump })
    }
}
mod kw {
    use syn::custom_keyword;
    custom_keyword!(peek);
    custom_keyword!(dump);
}

enum ParseAttributeArg {
    Peek(PeekArg),
    Dump(kw::dump),
}
impl Parse for ParseAttributeArg {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(kw::peek) && input.peek2(Paren) {
            Ok(Self::Peek(input.parse()?))
        } else if input.peek(kw::dump) {
            Ok(Self::Dump(input.parse()?))
        } else {
            Err(input.error("expected `peek(...)`"))
        }
    }
}
struct PeekArg {
    peek_token: kw::peek,
    paren_token: Paren,
    args: Punctuated<LitStr, Token![,]>,
}
impl Parse for PeekArg {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let peek_token = input.parse()?;
        let paren_token = parenthesized!(content in input);
        let args = content.parse_terminated(Parse::parse)?;
        Ok(Self {
            peek_token,
            paren_token,
            args,
        })
    }
}
impl ToTokens for PeekArg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.peek_token.to_tokens(tokens);
        self.paren_token.surround(tokens, |tokens| {
            self.args.to_tokens(tokens);
        });
    }
}
