use crate::syn_utils::*;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{parse_quote, spanned::Spanned, Data, DataEnum, DataStruct, DeriveInput, Fields, Result};

pub fn derive_to_tokens(input: DeriveInput) -> Result<TokenStream> {
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
    let ts = impl_trait_result(&input, &parse_quote!(::quote::ToTokens), &[], ts, false)?;
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
                let var_ident = to_var_ident(&field.ident, Some(index));
                vars.push(quote!(#var_ident));
            }
            quote!( #self_path( #(#vars,)*))
        }
        Fields::Named(_) => {
            for field in fields.iter() {
                let field_ident = &field.ident;
                let var_ident = to_var_ident(&field_ident, None);
                vars.push(quote!(#field_ident : #var_ident));
            }
            quote!( #self_path { #(#vars,)* } )
        }
    };
    Ok(ts)
}
fn code_from_fields(fields: &Fields) -> Result<TokenStream> {
    let mut ts = TokenStream::new();
    for (index, field) in fields.iter().enumerate() {
        let ident = to_var_ident(&field.ident, Some(index));
        ts.extend(quote_spanned!(field.span()=> ::quote::ToTokens::to_tokens(#ident, tokens); ));
    }
    Ok(ts)
}
fn to_var_ident(ident: &Option<Ident>, index: Option<usize>) -> Ident {
    if let Some(ident) = ident {
        format_ident!("_{}", ident)
    } else {
        format_ident!("_{}", index.unwrap())
    }
}
