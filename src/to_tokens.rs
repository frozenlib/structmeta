use crate::syn_utils::*;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use std::unreachable;
use syn::{
    parenthesized, parse::Parse, parse2, parse_quote, punctuated::Punctuated, spanned::Spanned,
    Data, DataEnum, DataStruct, DeriveInput, Field, Fields, LitStr, Result, Token,
};

pub fn derive_to_tokens(input: DeriveInput) -> Result<TokenStream> {
    let mut dump = false;
    for attr in &input.attrs {
        if attr.path.is_ident("to_tokens") {
            let attr: ToTokensAttribute = parse2(attr.tokens.clone())?;
            dump = dump || attr.dump.is_some();
        }
    }

    let ts = match &input.data {
        Data::Struct(data) => code_from_struct(&data)?,
        Data::Enum(data) => code_from_enum(&data)?,
        Data::Union(_) => {
            bail!("Not supported for union.")
        }
    };
    let ts = quote! {
        fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
            #ts
        }
    };
    let ts = impl_trait_result(&input, &parse_quote!(::quote::ToTokens), &[], ts, dump)?;
    Ok(ts)
}
fn code_from_struct(data: &DataStruct) -> Result<TokenStream> {
    let p = to_pattern(quote!(Self), &data.fields)?;
    let ts = code_from_fields(&data.fields)?;
    let ts = quote! {
        let #p = self;
        #ts
    };
    Ok(ts)
}
fn code_from_enum(data: &DataEnum) -> Result<TokenStream> {
    let mut arms = Vec::new();
    for variant in &data.variants {
        let ident = &variant.ident;
        let p = to_pattern(quote!(Self::#ident), &variant.fields)?;
        let code = code_from_fields(&variant.fields)?;
        arms.push(quote! {
            #p => {
                #code
            }
        });
    }
    Ok(quote! {
        match self {
            #(#arms)*
        }
    })
}
fn to_pattern(self_path: TokenStream, fields: &Fields) -> Result<TokenStream> {
    let mut vars = Vec::new();
    let ts = match fields {
        Fields::Unit => self_path,
        Fields::Unnamed(_) => {
            for (index, field) in fields.iter().enumerate() {
                let var_ident = to_var_ident(Some(index), &field.ident);
                vars.push(quote!(#var_ident));
            }
            quote!( #self_path( #(#vars,)*))
        }
        Fields::Named(_) => {
            for field in fields.iter() {
                let field_ident = &field.ident;
                let var_ident = to_var_ident(None, &field_ident);
                vars.push(quote!(#field_ident : #var_ident));
            }
            quote!( #self_path { #(#vars,)* } )
        }
    };
    Ok(ts)
}
struct Scope<'a> {
    ts: TokenStream,
    surround: Option<Surround<'a>>,
}
struct Surround<'a> {
    ident: Ident,
    field: &'a Field,
    close: char,
}
impl<'a> Scope<'a> {
    fn new(surround: Option<Surround<'a>>) -> Self {
        Self {
            ts: TokenStream::new(),
            surround,
        }
    }
}
impl<'a> Scope<'a> {
    fn into_code(self, close: Option<char>) -> Option<TokenStream> {
        if let Some(s) = self.surround {
            let mut mismatch = false;
            if let Some(close) = close {
                mismatch = close != s.close;
            }
            if !mismatch {
                let ident = &s.ident;
                let ts = self.ts;
                return Some(quote_spanned!(s.field.span()=>
                #ident.surround(tokens, |tokens| { #ts });
                ));
            }
        }
        None
    }
}
fn to_close(c: char) -> char {
    match c {
        '(' => ')',
        '[' => ']',
        '{' => '}',
        _ => panic!("not found closing delimiter for {}", c),
    }
}
fn code_from_fields(fields: &Fields) -> Result<TokenStream> {
    let mut scopes = vec![Scope::new(None)];
    for (index, field) in fields.iter().enumerate() {
        let ident = to_var_ident(Some(index), &field.ident);
        let mut field_to_tokens = true;
        for attr in &field.attrs {
            if attr.path.is_ident("to_tokens") {
                let attr: ToTokensAttribute = parse2(attr.tokens.clone())?;
                for token in &attr.token {
                    for c in token.value().chars() {
                        match c {
                            '(' | '[' | '{' => {
                                let close = to_close(c);
                                scopes.push(Scope::new(Some(Surround {
                                    ident: ident.clone(),
                                    field,
                                    close,
                                })));
                                field_to_tokens = false;
                            }
                            ')' | ']' | '}' => {
                                let scope = scopes.pop().unwrap();
                                if let Some(code) = scope.into_code(Some(c)) {
                                    scopes.last_mut().unwrap().ts.extend(code);
                                } else {
                                    bail!(token.span()=> "mismatched closing delimiter.");
                                }
                            }
                            _ => {
                                bail!(token.span() => "expected '(', ')', '[', ']', '{{' or '}}', found `{}`.", c);
                            }
                        }
                    }
                }
            }
        }
        if field_to_tokens {
            let code = quote_spanned!(field.span()=> ::quote::ToTokens::to_tokens(#ident, tokens););
            scopes.last_mut().unwrap().ts.extend(code);
        }
    }
    while let Some(scope) = scopes.pop() {
        if scopes.is_empty() {
            return Ok(scope.ts);
        }
        let code = scope.into_code(None).unwrap();
        scopes.last_mut().unwrap().ts.extend(code);
    }
    unreachable!()
}
fn to_var_ident(index: Option<usize>, ident: &Option<Ident>) -> Ident {
    if let Some(ident) = ident {
        format_ident!("_{}", ident)
    } else {
        format_ident!("_{}", index.unwrap())
    }
}

struct ToTokensAttribute {
    dump: Option<Span>,
    token: Vec<LitStr>,
}
impl Parse for ToTokensAttribute {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let content;
        parenthesized!(content in input);
        let args: Punctuated<ToTokensAttributeArg, Token![,]> =
            content.parse_terminated(ToTokensAttributeArg::parse)?;
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
        Ok(Self { token, dump })
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
