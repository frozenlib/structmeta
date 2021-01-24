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
    Data, DataEnum, DataStruct, DeriveInput, Expr, Fields, Ident, Result, Token,
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
    code_from_fields(quote!(Self), &data.fields, None)
}
fn code_from_enum(self_ident: &Ident, data: &DataEnum) -> Result<TokenStream> {
    let mut ts = TokenStream::new();
    for variant in &data.variants {
        let variant_ident = &variant.ident;
        let fn_ident = format_ident!("_parse_{}", &variant.ident);
        let mut peeks = Vec::new();
        let fn_expr = code_from_fields(
            quote!(#self_ident::#variant_ident),
            &variant.fields,
            Some(&mut peeks),
        )?;
        let fn_def = quote! {
            #[allow(non_snake_case)]
            fn #fn_ident(input: ::syn::parse::ParseStream<'_>) -> ::syn::Result<#self_ident> {
                #fn_expr
            }
        };
        let code = if peeks.is_empty() {
            quote! {
                let fork = input.fork();
                if let Ok(value) = #fn_ident(&fork) {
                    ::syn::parse::discouraged::Speculative::advance_to(input, &fork);
                    return Ok(value);
                }
            }
        } else {
            let mut preds = Vec::new();
            for (index, peek) in peeks.into_iter().enumerate() {
                preds.push(to_predicate(index, &peek)?);
            }
            quote! {
                if #(#preds )&&* {
                    return #fn_ident(&input);
                }
            }
        };
        ts.extend(quote! {
            #fn_def
            #code
        });
    }
    ts.extend(quote! {
        Err(input.error("parse failed."))
    });
    Ok(ts)
}
fn to_predicate(index: usize, peek: &PeekItem) -> Result<TokenStream> {
    let peek_ident: Ident = match index {
        0 => parse_quote!(peek),
        1 => parse_quote!(peek2),
        2 => parse_quote!(peek3),
        _ => bail!(peek.span, "more than three `#[parse(peek)]` was specified."),
    };
    let peek_arg = &peek.expr;
    Ok(quote!(input.#peek_ident(#peek_arg)))
}

struct Scope {
    input: Ident,
    close: Option<char>,
}
struct PeekItem {
    span: Span,
    expr: TokenStream,
}
fn to_parse_bracket(c: char) -> Ident {
    match c {
        '(' => parse_quote!(parenthesized),
        '[' => parse_quote!(bracketed),
        '{' => parse_quote!(braced),
        _ => unreachable!(),
    }
}
fn code_from_fields(
    self_path: TokenStream,
    fields: &Fields,
    mut peeks: Option<&mut Vec<PeekItem>>,
) -> Result<TokenStream> {
    let mut scopes = vec![Scope {
        input: parse_quote!(input),
        close: None,
    }];
    let mut ts = TokenStream::new();
    let mut inits = Vec::new();
    let mut non_peek_field = None;
    for (index, field) in fields.iter().enumerate() {
        let var_ident = to_var_ident(index, &field.ident);
        let mut use_parse = true;
        for attr in &field.attrs {
            let mut is_root = scopes.len() == 1;
            if attr.path.is_ident("to_tokens") {
                let attr: ToTokensAttribute = parse2(attr.tokens.clone())?;
                for token in attr.token {
                    for c in token.value().chars() {
                        match c {
                            '(' | '[' | '{' => {
                                use_parse = false;
                                let parse_bracket = to_parse_bracket(c);
                                let input_old = &scopes.last().unwrap().input;
                                let input = format_ident!("input{}", var_ident);
                                let code = quote_spanned!(field.span()=>
                                    let #input;
                                    let #var_ident = ::syn::#parse_bracket!(#input in #input_old);
                                    let #input = &#input;
                                );
                                ts.extend(code);
                                scopes.push(Scope {
                                    close: Some(to_close(c)),
                                    input,
                                });
                            }
                            ')' | ']' | '}' => {
                                if scopes.last().unwrap().close != Some(c) {
                                    bail!(token.span(), "mismatched closing delimiter `{}`.", c);
                                }
                                scopes.pop();
                                if scopes.len() == 1 {
                                    is_root = true;
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
            let mut is_peek = false;
            if attr.path.is_ident("parse") {
                let attr: ParseAttribute = parse2(attr.tokens.clone())?;
                if let Some(peek) = attr.peek {
                    if let Some(peeks) = &mut peeks {
                        let span = peek.span();
                        if !is_root {
                            bail!(span, "`peek` cannot be specified in [], () or {{}}.");
                        }
                        if let Some(non_peek_field) = &non_peek_field {
                            bail!(
                                span,
                                "you need to peek all previous tokens. consider specifying `#[parse(peek)]` for field `{}`.",
                                non_peek_field
                            );
                        }
                        let expr = match peek {
                            PeekArg::NoToken { .. } => field.ty.to_token_stream(),
                            PeekArg::WithToken { expr, .. } => expr.to_token_stream(),
                        };
                        peeks.push(PeekItem { span, expr });
                        is_peek = true;
                    }
                }
            }
            if is_root && !is_peek && non_peek_field.is_none() {
                non_peek_field = Some(to_display(index, &field.ident));
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

fn to_var_ident(index: usize, ident: &Option<Ident>) -> Ident {
    if let Some(ident) = ident {
        format_ident!("_{}", ident)
    } else {
        format_ident!("_{}", index)
    }
}
fn to_display(index: usize, ident: &Option<Ident>) -> String {
    if let Some(ident) = ident {
        format!("{}", ident)
    } else {
        format!("{}", index)
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
                ParseAttributeArg::Peek(peek_value) => {
                    if peek.is_some() {
                        bail!(peek_value.span(), "peek(..) is already specified.");
                    }
                    peek = Some(peek_value);
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
        if input.peek(kw::peek) {
            Ok(Self::Peek(input.parse()?))
        } else if input.peek(kw::dump) {
            Ok(Self::Dump(input.parse()?))
        } else {
            Err(input.error("expected `peek(...)`"))
        }
    }
}
enum PeekArg {
    NoToken {
        peek_token: kw::peek,
    },
    WithToken {
        peek_token: kw::peek,
        paren_token: Paren,
        expr: Expr,
    },
}
impl Parse for PeekArg {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let peek_token = input.parse()?;
        if input.peek(Paren) {
            let paren_token = parenthesized!(content in input);
            let expr = content.parse()?;
            Ok(Self::WithToken {
                peek_token,
                paren_token,
                expr,
            })
        } else {
            Ok(Self::NoToken { peek_token })
        }
    }
}
impl ToTokens for PeekArg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::NoToken { peek_token } => {
                peek_token.to_tokens(tokens);
            }
            Self::WithToken {
                peek_token,
                paren_token,
                expr,
            } => {
                peek_token.to_tokens(tokens);
                paren_token.surround(tokens, |tokens| {
                    expr.to_tokens(tokens);
                });
            }
        }
    }
}
