use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    punctuated::Punctuated, DeriveInput, Path, PathArguments, PathSegment, Result, Token, Type,
    WherePredicate,
};

macro_rules! bail {
    ($span:expr, $message:literal $(,)?) => {
        return std::result::Result::Err(syn::Error::new($span, $message))
    };
    ($span:expr, $err:expr $(,)?) => {
        return std::result::Result::Err(syn::Error::new($span, $err))
    };
    ($span:expr, $fmt:expr, $($arg:tt)*) => {
        return std::result::Result::Err(syn::Error::new($span, std::format!($fmt, $($arg)*)))
    };
}

pub fn into_macro_output(input: Result<TokenStream>) -> proc_macro::TokenStream {
    match input {
        Ok(s) => s,
        Err(e) => e.to_compile_error(),
    }
    .into()
}

pub fn impl_trait(
    input: &DeriveInput,
    trait_path: &Path,
    wheres: &[WherePredicate],
    contents: TokenStream,
) -> TokenStream {
    let ty = &input.ident;
    let (impl_g, ty_g, where_clause) = input.generics.split_for_impl();
    let mut wheres = wheres.to_vec();
    if let Some(where_clause) = where_clause {
        wheres.extend(where_clause.predicates.iter().cloned());
    }
    let where_clause = if wheres.is_empty() {
        quote! {}
    } else {
        quote! { where #(#wheres,)*}
    };
    quote! {
        #[automatically_derived]
        impl #impl_g #trait_path for #ty #ty_g #where_clause {
            #contents
        }
    }
}
pub fn impl_trait_result(
    input: &DeriveInput,
    trait_path: &Path,
    wheres: &[WherePredicate],
    contents: TokenStream,
    dump: bool,
) -> Result<TokenStream> {
    let ts = impl_trait(input, trait_path, wheres, contents);
    if dump {
        panic!("macro result: \n{ts}");
    }
    Ok(ts)
}

pub fn is_type(ty: &Type, ns: &[&[&str]], name: &str) -> bool {
    if let Some(a) = get_arguments_of(ty, ns, name) {
        a.is_empty()
    } else {
        false
    }
}
pub fn get_arguments_of<'a>(ty: &'a Type, ns: &[&[&str]], name: &str) -> Option<&'a PathArguments> {
    if let Type::Path(ty) = ty {
        if ty.qself.is_some() {
            return None;
        }
        let ss = &ty.path.segments;
        if let Some(last) = ty.path.segments.last() {
            if last.ident != name {
                return None;
            }
            return if ns.iter().any(|ns| is_match_ns(ss, ns)) {
                Some(&last.arguments)
            } else {
                None
            };
        }
    }
    None
}
pub fn is_match_ns(ss: &Punctuated<PathSegment, Token![::]>, ns: &[&str]) -> bool {
    let mut i_ss = ss.len() - 1;
    let mut i_ns = ns.len();
    while i_ss > 0 && i_ns > 0 {
        i_ns -= 1;
        i_ss -= 1;
        let s = &ss[i_ss];
        if s.ident != ns[i_ns] || !s.arguments.is_empty() {
            return false;
        }
    }
    i_ss == 0
}
pub const NS_SYN: &[&[&str]] = &[&["syn"]];

pub fn is_macro_delimiter(ty: &Type) -> bool {
    is_type(ty, NS_SYN, "MacroDelimiter")
}
