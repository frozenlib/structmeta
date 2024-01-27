use crate::{syn_utils::*, to_tokens_attribute::*};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use std::unreachable;
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    spanned::Spanned,
    Data, DataEnum, DataStruct, DeriveInput, Fields, Ident, Result, Token,
};

pub fn derive_parse(input: DeriveInput) -> Result<TokenStream> {
    let mut dump = false;
    for attr in &input.attrs {
        if attr.path().is_ident("parse") {
            let attr: ParseAttribute = attr.parse_args()?;
            dump = dump || attr.dump.is_some();
        }
    }

    let ts = match &input.data {
        Data::Struct(data) => code_from_struct(data)?,
        Data::Enum(data) => code_from_enum(&input.ident, data)?,
        Data::Union(_) => {
            bail!(Span::call_site(), "Not supported for union.")
        }
    };
    let ts = quote! {
        fn parse(input: ::structmeta::helpers::exports::syn::parse::ParseStream<'_>) -> ::structmeta::helpers::exports::syn::Result<Self> {
            #ts
        }
    };
    let ts = impl_trait_result(
        &input,
        &parse_quote!(::structmeta::helpers::exports::syn::parse::Parse),
        &[],
        ts,
        dump,
    )?;
    Ok(ts)
}

fn code_from_struct(data: &DataStruct) -> Result<TokenStream> {
    code_from_fields(quote!(Self), &data.fields, None)
}
fn code_from_enum(self_ident: &Ident, data: &DataEnum) -> Result<TokenStream> {
    let mut ts = TokenStream::new();
    ts.extend(quote!(
        use syn::ext::*;
    ));
    let mut input_is_forked = false;
    let mut input_is_moved = false;
    for index in 0..data.variants.len() {
        let variant = &data.variants[index];
        let variant_ident = &variant.ident;
        let is_last = index == data.variants.len() - 1;
        let fn_ident = format_ident!("_parse_{}", &variant.ident);
        let mut peeks = Vec::new();
        let fn_expr = code_from_fields(
            quote!(#self_ident::#variant_ident),
            &variant.fields,
            Some(&mut peeks),
        )?;
        let fn_def = quote! {
            #[allow(non_snake_case)]
            fn #fn_ident(input: ::structmeta::helpers::exports::syn::parse::ParseStream<'_>) -> ::structmeta::helpers::exports::syn::Result<#self_ident> {
                #fn_expr
            }
        };
        let code = if peeks.is_empty() {
            if is_last && !input_is_forked {
                input_is_moved = true;
                quote! {
                    #fn_ident(input)
                }
            } else {
                input_is_forked = true;
                quote! {
                    let fork = input.fork();
                    if let Ok(value) = #fn_ident(&fork) {
                        ::structmeta::helpers::exports::syn::parse::discouraged::Speculative::advance_to(input, &fork);
                        return Ok(value);
                    }
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
    if !input_is_moved {
        ts.extend(quote! {
            Err(input.error("parse failed."))
        });
    }
    Ok(ts)
}
fn to_predicate(index: usize, peek: &PeekItem) -> Result<TokenStream> {
    let peek_ident: Ident = match index {
        0 => parse_quote!(peek),
        1 => parse_quote!(peek2),
        2 => parse_quote!(peek3),
        _ => bail!(peek.span, "more than three `#[parse(peek)]` was specified."),
    };
    let peek_arg = &peek.arg;
    Ok(quote!(input.#peek_ident(#peek_arg)))
}

struct Scope {
    input: Ident,
    close: Option<char>,
}
struct PeekItem {
    span: Span,
    arg: TokenStream,
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
        let ty = &field.ty;
        let var_ident = to_var_ident(index, &field.ident);
        let mut use_parse = true;
        let mut peek = None;
        let mut is_any = false;
        let mut is_terminated = false;
        let mut is_root = scopes.len() == 1;
        for attr in &field.attrs {
            if attr.path().is_ident("to_tokens") {
                let attr: ToTokensAttribute = attr.parse_args()?;
                for token in attr.token {
                    for c in token.value().chars() {
                        match c {
                            '(' | '[' | '{' => {
                                use_parse = false;
                                let parse_bracket = if is_macro_delimiter(&field.ty) {
                                    quote!(::structmeta::helpers_parse_macro_delimiter)
                                } else {
                                    let parse_bracket = to_parse_bracket(c);
                                    quote!(::structmeta::helpers::exports::syn::#parse_bracket)
                                };
                                let input_old = &scopes.last().unwrap().input;
                                let input = format_ident!("input_{}", index);
                                let ty = &field.ty;
                                let code = quote_spanned!(field.span()=>
                                    let #input;
                                    let #var_ident = #parse_bracket!(#input in #input_old);
                                    let #var_ident : #ty = #var_ident;
                                    let #input = &#input;
                                );
                                // We need `let #var_ident : #ty = #var_ident;` to make error messages easier to understand.
                                // Try remove this line and run compile fail tests for details.

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
            if attr.path().is_ident("parse") {
                let attr: ParseAttribute = attr.parse_args()?;
                peek = peek.or(attr.peek);
                is_any = is_any || attr.any.is_some();
                is_terminated = is_terminated || attr.terminated.is_some();
            }
        }
        if let Some(peeks) = &mut peeks {
            if let Some(peek) = peek {
                let span = peek.span();
                if !is_root {
                    bail!(span, "`#[parse(peek)]` cannot be specified with a field enclosed by `[]`, `()` or `{}`.");
                }
                if let Some(non_peek_field) = &non_peek_field {
                    bail!(
                            span,
                            "you need to peek all previous tokens. consider specifying `#[parse(peek)]` for field `{}`.",
                            non_peek_field
                        );
                }
                let arg = if is_any {
                    quote!(#ty::peek_any)
                } else {
                    quote!(#ty)
                };
                peeks.push(PeekItem { span, arg });
            }
        }
        if is_root && peek.is_none() && non_peek_field.is_none() {
            non_peek_field = Some(to_display(index, &field.ident));
        }
        if use_parse {
            let input = &scopes.last().unwrap().input;
            let expr = match (is_terminated, is_any) {
                (false, false) => {
                    quote!(::structmeta::helpers::exports::syn::parse::Parse::parse(#input))
                }
                (false, true) => quote!(<#ty>::parse_any(#input)),
                (true, false) => {
                    quote!(<#ty>::parse_terminated(#input))
                }
                (true, true) => {
                    quote!(<#ty>::parse_terminated_with(#input, ::structmeta::helpers::exports::syn::ext::IdentExt::parse_any))
                }
            };
            let code = quote_spanned!(field.span()=>let #var_ident = #expr?;);
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
        use syn::ext::*;
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
        format!("{ident}")
    } else {
        format!("{index}")
    }
}

struct ParseAttribute {
    any: Option<kw::any>,
    peek: Option<kw::peek>,
    terminated: Option<kw::terminated>,
    dump: Option<kw::dump>,
}
impl Parse for ParseAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut any = None;
        let mut peek = None;
        let mut terminated = None;
        let mut dump = None;
        let args = input.parse_terminated(ParseAttributeArg::parse, Token![,])?;
        for arg in args.into_iter() {
            match arg {
                ParseAttributeArg::Any(kw_any) => any = any.or(Some(kw_any)),
                ParseAttributeArg::Peek(kw_peek) => peek = peek.or(Some(kw_peek)),
                ParseAttributeArg::Terminated(kw_terminated) => {
                    terminated = terminated.or(Some(kw_terminated))
                }
                ParseAttributeArg::Dump(kw_dump) => dump = dump.or(Some(kw_dump)),
            }
        }
        Ok(Self {
            any,
            peek,
            terminated,
            dump,
        })
    }
}
mod kw {
    use syn::custom_keyword;
    custom_keyword!(any);
    custom_keyword!(peek);
    custom_keyword!(terminated);
    custom_keyword!(dump);
}

enum ParseAttributeArg {
    Any(kw::any),
    Peek(kw::peek),
    Terminated(kw::terminated),
    Dump(kw::dump),
}
impl Parse for ParseAttributeArg {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(kw::any) {
            Ok(Self::Any(input.parse()?))
        } else if input.peek(kw::peek) {
            Ok(Self::Peek(input.parse()?))
        } else if input.peek(kw::terminated) {
            Ok(Self::Terminated(input.parse()?))
        } else if input.peek(kw::dump) {
            Ok(Self::Dump(input.parse()?))
        } else {
            Err(input.error("expected `any`, `peek`, `terminated` or `dump`."))
        }
    }
}
