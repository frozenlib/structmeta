use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Path, Result, WherePredicate};

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
    ($($tt:tt)*) => {
        bail!(proc_macro2::Span::call_site(), $($tt)*)
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
        panic!("macro result: \n{}", ts);
    }
    Ok(ts)
}
