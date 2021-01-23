use crate::syn_utils::*;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    token::Paren,
    Data, DataEnum, DataStruct, DeriveInput, Fields, Ident, LitStr, Result, Token,
};

pub fn derive_parse(input: DeriveInput) -> Result<TokenStream> {
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
    let ts = impl_trait_result(&input, &parse_quote!(::syn::parse::Parse), &[], ts, false)?;
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
fn code_from_fields(self_path: TokenStream, fields: &Fields) -> Result<TokenStream> {
    let mut vars = Vec::new();
    let mut inits = Vec::new();
    for (index, field) in fields.iter().enumerate() {
        let var_ident = to_var_ident(&field.ident, Some(index));
        vars.push(
            quote_spanned!(field.span()=>let #var_ident = ::syn::parse::Parse::parse(input)?;),
        );
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
        #(#vars)*
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

mod kw {
    use syn::custom_keyword;
    custom_keyword!(peek);
}

// struct ParseAttribute {
//     peek: Option<PeekArg>,
// }
// impl Parse for ParseAttribute {
//     fn parse(input: ParseStream) -> Result<Self> {
//         let content;
//         parenthesized!(content in input);
//         let mut peek = None;
//         let args: Punctuated<ParseAttributeArg, Token![,]> =
//             content.parse_terminated(ParseAttributeArg::parse)?;
//         for arg in args.into_iter() {
//             match arg {
//                 ParseAttributeArg::Peek(peek_arg) => {
//                     if peek.is_some() {
//                         bail!(peek_arg.span() => "peek(..) is already specified.");
//                     }
//                     peek = Some(peek_arg);
//                 }
//             }
//         }
//         Ok(Self { peek })
//     }
// }
enum ParseAttributeArg {
    Peek(PeekArg),
}
impl Parse for ParseAttributeArg {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(kw::peek) && input.peek2(Paren) {
            Ok(Self::Peek(PeekArg::parse(input)?))
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
