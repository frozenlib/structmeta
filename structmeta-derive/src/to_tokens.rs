use crate::{syn_utils::*, to_tokens_attribute::*};
use proc_macro2::{Delimiter, Ident, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use std::unreachable;
use syn::{
    parse_quote, spanned::Spanned, Data, DataEnum, DataStruct, DeriveInput, Field, Fields, Result,
};

pub fn derive_to_tokens(input: DeriveInput) -> Result<TokenStream> {
    let mut dump = false;
    for attr in &input.attrs {
        if attr.path().is_ident("to_tokens") {
            let attr: ToTokensAttribute = attr.parse_args()?;
            dump = dump || attr.dump.is_some();
        }
    }

    let ts = match &input.data {
        Data::Struct(data) => code_from_struct(data)?,
        Data::Enum(data) => code_from_enum(data)?,
        Data::Union(_) => {
            bail!(Span::call_site(), "Not supported for union.")
        }
    };
    let ts = quote! {
        fn to_tokens(&self, tokens: &mut ::structmeta::helpers::exports::proc_macro2::TokenStream) {
            #ts
        }
    };
    let ts = impl_trait_result(
        &input,
        &parse_quote!(::structmeta::helpers::exports::quote::ToTokens),
        &[],
        ts,
        dump,
    )?;
    Ok(ts)
}
fn code_from_struct(data: &DataStruct) -> Result<TokenStream> {
    let p = to_pattern(quote!(Self), &data.fields);
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
        let p = to_pattern(quote!(Self::#ident), &variant.fields);
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
fn to_pattern(self_path: TokenStream, fields: &Fields) -> TokenStream {
    let mut vars = Vec::new();
    match fields {
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
                let var_ident = to_var_ident(None, field_ident);
                vars.push(quote!(#field_ident : #var_ident));
            }
            quote!( #self_path { #(#vars,)* } )
        }
    }
}
struct Scope<'a> {
    ts: TokenStream,
    surround: Option<Surround<'a>>,
}
struct Surround<'a> {
    ident: Ident,
    field: &'a Field,
    delimiter: Delimiter,
}

fn delimiter_from_open_char(value: char) -> Option<Delimiter> {
    match value {
        '[' => Some(Delimiter::Bracket),
        '{' => Some(Delimiter::Brace),
        '(' => Some(Delimiter::Parenthesis),
        _ => None,
    }
}
fn delimiter_from_close_char(value: char) -> Option<Delimiter> {
    match value {
        ']' => Some(Delimiter::Bracket),
        '}' => Some(Delimiter::Brace),
        ')' => Some(Delimiter::Parenthesis),
        _ => None,
    }
}

impl<'a> Scope<'a> {
    fn new(surround: Option<Surround<'a>>) -> Self {
        Self {
            ts: TokenStream::new(),
            surround,
        }
    }
}
fn close_char_of(delimiter: Delimiter) -> char {
    match delimiter {
        Delimiter::Bracket => ']',
        Delimiter::Brace => '}',
        Delimiter::Parenthesis => ')',
        _ => unreachable!("unsupported delimiter"),
    }
}
impl<'a> Surround<'a> {
    fn token_type_ident(&self) -> Ident {
        match self.delimiter {
            Delimiter::Bracket => parse_quote!(Bracket),
            Delimiter::Brace => parse_quote!(Brace),
            Delimiter::Parenthesis => parse_quote!(Paren),
            _ => unreachable!("unsupported delimiter"),
        }
    }
}

impl<'a> Scope<'a> {
    fn into_code(self, delimiter: Option<Delimiter>, span: Span) -> Result<TokenStream> {
        if let Some(s) = self.surround {
            if let Some(delimiter) = delimiter {
                if s.delimiter != delimiter {
                    bail!(
                        span,
                        "mismatched closing delimiter expected `{}`, found `{}`.",
                        close_char_of(s.delimiter),
                        close_char_of(delimiter),
                    )
                }
            }
            let ident = &s.ident;
            let ts = self.ts;
            let ty = &s.field.ty;
            let span = s.field.span();
            let func = if is_macro_delimiter(ty) {
                quote_spanned!(span=> ::structmeta::helpers::surround_macro_delimiter)
            } else {
                let ty = s.token_type_ident();
                quote_spanned!(span=> ::structmeta::helpers::exports::syn::token::#ty::surround)
            };
            let code = quote_spanned!(span=> #func(#ident, tokens, |tokens| { #ts }););
            return Ok(code);
        }
        Ok(quote!())
    }
}
fn code_from_fields(fields: &Fields) -> Result<TokenStream> {
    let mut scopes = vec![Scope::new(None)];
    for (index, field) in fields.iter().enumerate() {
        let ident = to_var_ident(Some(index), &field.ident);
        let mut field_to_tokens = true;
        for attr in &field.attrs {
            if attr.path().is_ident("to_tokens") {
                let attr: ToTokensAttribute = attr.parse_args()?;
                for token in &attr.token {
                    for c in token.value().chars() {
                        if let Some(delimiter) = delimiter_from_open_char(c) {
                            scopes.push(Scope::new(Some(Surround {
                                ident: ident.clone(),
                                field,
                                delimiter,
                            })));
                            field_to_tokens = false;
                        } else if let Some(delimiter) = delimiter_from_close_char(c) {
                            let scope = scopes.pop().unwrap();
                            scopes
                                .last_mut()
                                .unwrap()
                                .ts
                                .extend(scope.into_code(Some(delimiter), token.span())?);
                        } else {
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
        if field_to_tokens {
            let code = quote_spanned!(field.span()=> ::structmeta::helpers::exports::quote::ToTokens::to_tokens(#ident, tokens););
            scopes.last_mut().unwrap().ts.extend(code);
        }
    }
    while let Some(scope) = scopes.pop() {
        if scopes.is_empty() {
            return Ok(scope.ts);
        }
        scopes
            .last_mut()
            .unwrap()
            .ts
            .extend(scope.into_code(None, Span::call_site())?);
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
