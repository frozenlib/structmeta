use crate::{syn_utils::*, to_tokens_attribute::*};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use std::unreachable;
use syn::{
    parse2, parse_quote, spanned::Spanned, Data, DataEnum, DataStruct, DeriveInput, Field, Fields,
    Result,
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
                let var_ident = to_var_ident(None, &field_ident);
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
    kind: SurroundKind,
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum SurroundKind {
    Bracket,
    Brace,
    Paren,
}
impl SurroundKind {
    fn close(&self) -> char {
        match self {
            SurroundKind::Bracket => ']',
            SurroundKind::Brace => '}',
            SurroundKind::Paren => ')',
        }
    }
    fn from_open(value: char) -> Option<Self> {
        match value {
            '[' => Some(Self::Bracket),
            '{' => Some(Self::Brace),
            '(' => Some(Self::Paren),
            _ => None,
        }
    }
    fn from_close(value: char) -> Option<Self> {
        match value {
            ']' => Some(Self::Bracket),
            '}' => Some(Self::Brace),
            ')' => Some(Self::Paren),
            _ => None,
        }
    }
    fn type_ident(&self) -> Ident {
        match self {
            SurroundKind::Bracket => parse_quote!(Bracket),
            SurroundKind::Brace => parse_quote!(Brace),
            SurroundKind::Paren => parse_quote!(Paren),
        }
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
impl<'a> Scope<'a> {
    fn into_code(self, surround_kind: Option<SurroundKind>) -> Option<TokenStream> {
        if let Some(s) = self.surround {
            let mut mismatch = false;
            if let Some(surround_kind) = surround_kind {
                mismatch = s.kind != surround_kind;
            }
            if !mismatch {
                let ty = s.kind.type_ident();
                let ident = &s.ident;
                let ts = self.ts;
                return Some(quote_spanned!(s.field.span()=>
                ::syn::token::#ty::surround(#ident, tokens, |tokens| { #ts });
                ));
            }
        }
        None
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
                        if let Some(kind) = SurroundKind::from_open(c) {
                            scopes.push(Scope::new(Some(Surround {
                                ident: ident.clone(),
                                field,
                                kind,
                            })));
                            field_to_tokens = false;
                        } else if let Some(kind) = SurroundKind::from_close(c) {
                            let scope = scopes.pop().unwrap();
                            if let Some(code) = scope.into_code(Some(kind)) {
                                scopes.last_mut().unwrap().ts.extend(code);
                            } else {
                                bail!(
                                    token.span(),
                                    "mismatched closing delimiter expected `{}`, found `{}`.",
                                    kind.close(),
                                    c
                                );
                            }
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
