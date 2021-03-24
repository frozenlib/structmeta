use crate::syn_utils::*;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use std::collections::HashMap;
use syn::{
    ext::IdentExt,
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    Data, DeriveInput, Field, Fields, GenericArgument, Ident, LitStr, PathArguments, PathSegment,
    Result, Token, Type,
};

pub fn derive_struct_meta(input: DeriveInput) -> Result<TokenStream> {
    if let Data::Struct(data) = &input.data {
        let mut dump = false;
        for attr in &input.attrs {
            if attr.path.is_ident("struct_meta") {
                let args = attr.parse_args_with(
                    Punctuated::<StructMetaAttributeArg, Token![,]>::parse_terminated,
                )?;
                for arg in args {
                    match arg {
                        StructMetaAttributeArg::Dump(_) => dump = true,
                    }
                }
            }
        }
        let ps = Params::from_fields(&data.fields)?;
        let body = ps.build();
        impl_trait_result(
            &input,
            &parse_quote!(::syn::parse::Parse),
            &[],
            quote! {
                fn parse(input: ::syn::parse::ParseStream<'_>) -> ::syn::Result<Self> {
                    #body
                }
            },
            dump,
        )
    } else {
        bail!(
            input.span(),
            "`#[derive(StructMeta)]` supports only struct."
        )
    }
}
struct Params<'a> {
    fields: &'a Fields,
    named: HashMap<String, NamedParam<'a>>,
    unnamed_required: Vec<UnnamedParam<'a>>,
    unnamed_optional: Vec<UnnamedParam<'a>>,
    unnamed_variadic: Option<UnnamedParam<'a>>,
}
impl<'a> Params<'a> {
    fn from_fields(fields: &'a Fields) -> Result<Self> {
        let mut unnamed_required = Vec::new();
        let mut unnamed_optional = Vec::new();
        let mut unnamed_variadic = None;
        let mut named = HashMap::new();
        for (index, field) in fields.iter().enumerate() {
            match Param::from_field(index, field)? {
                Param::Named(p) => {
                    if named.contains_key(&p.name) {
                        bail!(p.name_span, "`{}` is already exists.", p.name);
                    }
                    named.insert(p.name.clone(), p);
                }
                Param::Unnamed(p) => {
                    let span = field.span();
                    if unnamed_variadic.is_some() {
                        bail!(
                            span,
                            "cannot use unnamed parameter after variadic parameter."
                        )
                    }
                    if p.ty.is_vec {
                        unnamed_variadic = Some(p);
                    } else if p.info.is_option {
                        unnamed_optional.push(p);
                    } else {
                        if !unnamed_optional.is_empty() {
                            bail!(
                                span,
                                "cannot use non optional parameter after variadic parameter."
                            )
                        }
                        unnamed_required.push(p);
                    }
                }
            }
        }
        Ok(Self {
            fields,
            named,
            unnamed_required,
            unnamed_optional,
            unnamed_variadic,
        })
    }
    fn build(&self) -> TokenStream {
        let mut is_next = false;
        let mut ts = TokenStream::new();
        let mut ctor_args = vec![TokenStream::new(); self.fields.len()];
        for p in &self.unnamed_required {
            if is_next {
                ts.extend(quote!(input.parse::<::syn::Token![,]>()?;));
            }
            is_next = true;
            ts.extend(p.info.build_let_parse());
            p.build_ctor_arg(false, &mut ctor_args);
        }

        let mut arms_unnamed = Vec::new();
        let mut index = 0;
        for p in &self.unnamed_optional {
            ts.extend(p.info.build_let_none());
            arms_unnamed.push(p.build_arm_parse_value(index));
            p.build_ctor_arg(true, &mut ctor_args);
            index += 1;
        }
        if let Some(p) = &self.unnamed_variadic {
            ts.extend(p.info.build_let_vec_new());
            arms_unnamed.push(p.build_arm_parse_vec_item());
            p.build_ctor_arg(false, &mut ctor_args);
        } else {
            arms_unnamed.push(quote! {
                _ => { return Err(input.error("too many unnamed parameter")); }
            });
        }
        for p in self.named.values() {
            ts.extend(p.build_let());
            p.build_ctor_arg(&mut ctor_args);
        }
        let ps_flag = self.named_filter(|p| p.ty.is_flag());
        let ps_name_value = self.named_filter(|p| p.ty.is_name_value());
        let ps_name_args = self.named_filter(|p| p.ty.is_name_args());

        let mut arms_named = Vec::new();
        let mut index = 0;
        for p in &ps_flag {
            arms_named.push(p.build_arm_parse(index, ArgKind::Flag));
            index += 1;
        }
        for p in &ps_name_value {
            arms_named.push(p.build_arm_parse(index, ArgKind::NameValue));
            index += 1;
        }
        for p in &ps_name_args {
            arms_named.push(p.build_arm_parse(index, ArgKind::NameArgs));
            index += 1;
        }

        let names_flag = NamedParam::to_names(&ps_flag);
        let names_name_value = NamedParam::to_names(&ps_name_value);
        let names_name_args = NamedParam::to_names(&ps_name_args);
        let no_unnamed = self.unnamed_optional.is_empty() && self.unnamed_variadic.is_none();
        let ctor_args = match &self.fields {
            Fields::Named(_) => {
                quote!({ #(#ctor_args,)*})
            }
            Fields::Unnamed(_) => {
                quote!(( #(#ctor_args,)*))
            }
            Fields::Unit => {
                quote!()
            }
        };

        let ts_parse_unnamed = if !self.unnamed_optional.is_empty()
            || self.unnamed_variadic.is_some()
        {
            quote! {
                if named_used {
                    return Err(input.error("cannot use unnamed parameter after named parameter"));
                }
                match unnamed_index {
                    #(#arms_unnamed)*
                }
                unnamed_index += 1;
            }
        } else {
            quote! {
                return Err(input.error("cannot use unnamed parameter"));
            }
        };

        ts.extend(quote! {
            let mut is_next = #is_next;
            let mut unnamed_index = 0;
            let mut named_used = false;
            while !input.is_empty() {
                if is_next {
                    input.parse::<::syn::Token![,]>()?;
                    if input.is_empty() {
                        break;
                    }
                }
                is_next = true;
                if let Some((index, span)) = ::structmeta::helpers::try_parse_name(input,
                    &[#(#names_flag,)*],
                    &[#(#names_name_value,)*],
                    &[#(#names_name_args,)*],
                    #no_unnamed)?
                {
                    named_used = true;
                    match index {
                        #(#arms_named)*
                        _ => unreachable!()
                    }

                } else {
                    #ts_parse_unnamed
                }
            }
            Ok(Self #ctor_args)
        });

        ts
    }
    fn named_filter(&self, f: impl Fn(&&NamedParam<'a>) -> bool) -> Vec<&NamedParam<'a>> {
        self.named.values().filter(f).collect()
    }
}

enum Param<'a> {
    Named(NamedParam<'a>),
    Unnamed(UnnamedParam<'a>),
}

impl<'a> Param<'a> {
    fn from_field(index: usize, field: &'a Field) -> Result<Self> {
        let mut name = None;
        let mut unnamed = false;
        for attr in &field.attrs {
            if attr.path.is_ident("name") {
                if name.is_some() {
                    bail!(
                        attr.span(),
                        "`#[name(\"...\")]` can only be specified once."
                    )
                }
                let args: LitStr = attr.parse_args()?;
                name = Some((args.value(), args.span()));
            }
            if attr.path.is_ident("unnamed") {
                unnamed = true;
            }
        }
        if name.is_none() {
            if let Some(ident) = &field.ident {
                name = Some((ident.unraw().to_string(), ident.span()));
            }
        }
        if unnamed {
            name = None;
        }
        let (is_option, ty) = if let Some(ty) = get_option_element(&field.ty) {
            (true, ty)
        } else {
            (false, &field.ty)
        };
        let info = ParamInfo::new(index, field, is_option, ty);
        let ty = NamedParamType::from_type(ty);
        Ok(if let Some((name, name_span)) = name {
            Param::Named(NamedParam {
                info,
                ty,
                name,
                name_span,
            })
        } else {
            if let NamedParamType::Value { ty } = ty {
                Param::Unnamed(UnnamedParam { info, ty })
            } else {
                bail!(
                    info.span(),
                    "this field type cannot be used as unnamed parameter."
                )
            }
        })
    }
}

struct ParamInfo<'a> {
    index: usize,
    field: &'a Field,
    is_option: bool,
    ty: &'a Type,
    temp_ident: Ident,
}
impl<'a> ParamInfo<'a> {
    fn new(index: usize, field: &'a Field, is_option: bool, ty: &'a Type) -> Self {
        let temp_ident = format_ident!("_value_{}", index);
        Self {
            index,
            field,
            is_option,
            ty,
            temp_ident,
        }
    }
    fn span(&self) -> Span {
        self.field.span()
    }
    fn build_let_none(&self) -> TokenStream {
        let temp_ident = &self.temp_ident;
        let ty = &self.ty;
        quote!(let mut #temp_ident : Option<#ty> = None;)
    }
    fn build_let_vec_new(&self) -> TokenStream {
        let temp_ident = &self.temp_ident;
        let ty = &self.ty;
        quote!(let mut #temp_ident = <#ty>::new();)
    }
    fn build_let_parse(&self) -> TokenStream {
        let temp_ident = &self.temp_ident;
        let ty = &self.field.ty;
        quote_spanned!(self.span()=> let #temp_ident = input.parse::<#ty>()?;)
    }
}

struct NamedParam<'a> {
    info: ParamInfo<'a>,
    name_span: Span,
    name: String,
    ty: NamedParamType<'a>,
}

struct UnnamedParam<'a> {
    info: ParamInfo<'a>,
    ty: ValueParamType<'a>,
}
impl<'a> NamedParam<'a> {
    fn build_let(&self) -> TokenStream {
        let temp_ident = &self.info.temp_ident;
        quote!(let mut #temp_ident = None;)
    }
    fn build_arm_parse(&self, index: usize, kind: ArgKind) -> TokenStream {
        let temp_ident = &self.info.temp_ident;
        let msg = format!("parameter `{}` speficied more than once", self.name);
        let expr = self.ty.build_parse_expr(kind);
        quote_spanned! { self.info.field.span()=>
            #index => {
                if #temp_ident.is_some() {
                    return Err(::syn::Error::new(span, #msg));
                }
                #temp_ident = Some(#expr);
            }
        }
    }
    fn to_names<'b>(ps: &[&'b Self]) -> Vec<&'b str> {
        ps.into_iter().map(|x| x.name.as_str()).collect()
    }
    fn build_ctor_arg(&self, ctor_args: &mut [TokenStream]) {
        let temp_ident = &self.info.temp_ident;
        let value = if self.info.is_option {
            quote!(#temp_ident)
        } else {
            match self.ty {
                NamedParamType::Flag => quote!(::structmeta::Flag { span: #temp_ident }),
                NamedParamType::Bool => quote!(#temp_ident.is_some()),
                NamedParamType::Value { .. }
                | NamedParamType::NameValue { .. }
                | NamedParamType::NameArgs { .. } => {
                    let msg = format!("missing argument `{}`", self.name);
                    quote!(#temp_ident.ok_or_else(|| ::syn::Error::new(::proc_macro2::Span::call_site(), #msg))?)
                }
            }
        };
        build_ctor_arg(&self.info, value, ctor_args)
    }
}
impl<'a> UnnamedParam<'a> {
    fn build_arm_parse_value(&self, index: usize) -> TokenStream {
        let temp_ident = &self.info.temp_ident;
        let expr = self.ty.build_parse_expr(ArgKind::Value);
        quote_spanned! { self.info.field.span()=>
            #index => {
                #temp_ident = Some(#expr);
            }
        }
    }
    fn build_arm_parse_vec_item(&self) -> TokenStream {
        let temp_ident = &self.info.temp_ident;
        let expr = self.ty.build_parse_expr(ArgKind::Value);
        quote_spanned! { self.info.field.span()=>
            _ => {
                #temp_ident.push(#expr);
            }
        }
    }
    fn build_ctor_arg(&self, var_is_option: bool, ctor_args: &mut [TokenStream]) {
        let temp_ident = &self.info.temp_ident;
        let value = match (var_is_option, self.info.is_option) {
            (false, false) | (true, true) => {
                quote!(#temp_ident)
            }
            (true, false) => {
                quote!(#temp_ident.unwrap())
            }
            _ => {
                unreachable!()
            }
        };
        build_ctor_arg(&self.info, value, ctor_args)
    }
}
fn build_ctor_arg(info: &ParamInfo, value: TokenStream, ctor_args: &mut [TokenStream]) {
    let value = if let Some(ident) = &info.field.ident {
        quote!(#ident : #value)
    } else {
        value
    };
    ctor_args[info.index] = value;
}

mod kw {
    use syn::custom_keyword;

    custom_keyword!(dump);
}

enum StructMetaAttributeArg {
    Dump(kw::dump),
}
impl Parse for StructMetaAttributeArg {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(kw::dump) {
            return Ok(Self::Dump(input.parse()?));
        }
        Err(input.error("usage : #[struct_meta(dump)]"))
    }
}

enum NamedParamType<'a> {
    Bool,
    Flag,
    Value {
        ty: ValueParamType<'a>,
    },
    NameValue {
        ty: &'a Type,
        is_option: bool,
    },
    NameArgs {
        ty: ValueParamType<'a>,
        is_option: bool,
    },
}
struct ValueParamType<'a> {
    ty: &'a Type,
    is_vec: bool,
}

impl<'a> NamedParamType<'a> {
    fn from_type(ty: &'a Type) -> Self {
        if is_bool(ty) {
            Self::Bool
        } else if is_flag(ty) {
            Self::Flag
        } else if let Some(ty) = get_name_value_element(ty) {
            let (ty, is_option) = if let Some(ty) = get_option_element(ty) {
                (ty, true)
            } else {
                (ty, false)
            };
            Self::NameValue { ty, is_option }
        } else if let Some(ty) = get_name_args_element(ty) {
            let (ty, is_option) = if let Some(ty) = get_option_element(ty) {
                (ty, true)
            } else {
                (ty, false)
            };
            Self::NameArgs {
                ty: ValueParamType::from_type(ty),
                is_option,
            }
        } else {
            Self::Value {
                ty: ValueParamType::from_type(ty),
            }
        }
    }
    fn is_flag(&self) -> bool {
        match self {
            NamedParamType::Bool | NamedParamType::Flag => true,
            NamedParamType::Value { .. } => false,
            NamedParamType::NameValue { is_option, .. } => *is_option,
            NamedParamType::NameArgs { is_option, .. } => *is_option,
        }
    }
    fn is_name_value(&self) -> bool {
        match self {
            NamedParamType::Bool | NamedParamType::Flag => false,
            NamedParamType::Value {
                ty: ValueParamType { is_vec, .. },
            } => !is_vec,
            NamedParamType::NameValue { .. } => true,
            NamedParamType::NameArgs { .. } => false,
        }
    }
    fn is_name_args(&self) -> bool {
        match self {
            NamedParamType::Bool | NamedParamType::Flag => false,
            NamedParamType::Value {
                ty: ValueParamType { is_vec, .. },
            } => *is_vec,
            NamedParamType::NameValue { .. } => false,
            NamedParamType::NameArgs { .. } => true,
        }
    }
    fn build_parse_expr(&self, kind: ArgKind) -> TokenStream {
        match self {
            NamedParamType::Bool | NamedParamType::Flag => quote!(span),
            NamedParamType::Value { ty } => ty.build_parse_expr(kind),
            NamedParamType::NameValue { ty, is_option } => {
                let value = if kind == ArgKind::Flag && *is_option {
                    quote!(None)
                } else if *is_option {
                    quote!(Some(input.parse::<#ty>()?))
                } else {
                    quote!(input.parse::<#ty>()?)
                };
                quote!(::structmeta::NameValue { span, value: #value })
            }
            NamedParamType::NameArgs { ty, is_option } => {
                let args = ty.build_parse_expr(kind);
                let args = if kind == ArgKind::Flag && *is_option {
                    quote!(None)
                } else if *is_option {
                    quote!(Some(#args))
                } else {
                    args
                };
                quote!(structmeta::NameArgs { span, args: #args })
            }
        }
    }
}
impl<'a> ValueParamType<'a> {
    fn from_type(ty: &'a Type) -> Self {
        let (is_vec, ty) = if let Some(ty) = get_vec_element(ty) {
            (true, ty)
        } else {
            (false, ty)
        };
        Self { is_vec, ty }
    }
    fn build_parse_expr(&self, kind: ArgKind) -> TokenStream {
        let ty = self.ty;
        if self.is_vec {
            match kind {
                ArgKind::Flag => unreachable!(),
                ArgKind::Value => quote!(input.parse::<#ty>()?),
                ArgKind::NameValue => quote!(input.parse::<::std::vec::Vec<#ty>>()?),
                ArgKind::NameArgs => quote! {
                    {
                        let content;
                        ::syn::parenthesized!(content in input);
                        ::syn::punctuated::Punctuated::<#ty, ::syn::Token![,]>::parse_terminated(&content)?.into_iter().collect()
                    }
                },
            }
        } else {
            match kind {
                ArgKind::Flag => unreachable!(),
                ArgKind::Value | ArgKind::NameValue => quote!(input.parse::<#ty>()?),
                ArgKind::NameArgs => quote! {
                    {
                        let content;
                        ::syn::parenthesized!(content in input);
                        content.parse::<#ty>()?
                    }
                },
            }
        }
    }
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
enum ArgKind {
    Flag,
    Value,
    NameValue,
    NameArgs,
}

fn get_option_element(ty: &Type) -> Option<&Type> {
    get_element(ty, &[&["std", "option"], &["core", "option"]], "Option")
}
fn get_vec_element(ty: &Type) -> Option<&Type> {
    get_element(ty, &[&["std", "vec"], &["alloc", "vec"]], "Vec")
}
fn get_name_value_element(ty: &Type) -> Option<&Type> {
    get_element(ty, NS_STRUCTMETA, "NameValue")
}
fn get_name_args_element(ty: &Type) -> Option<&Type> {
    get_element(ty, NS_STRUCTMETA, "NameArgs")
}
fn is_bool(ty: &Type) -> bool {
    is_type(ty, NS_PRIMITIVE, "bool")
}
fn is_flag(ty: &Type) -> bool {
    is_type(ty, NS_STRUCTMETA, "Flag")
}

fn get_element<'a>(ty: &'a Type, ns: &[&[&str]], name: &str) -> Option<&'a Type> {
    if let PathArguments::AngleBracketed(args) = get_argumnets_of(ty, ns, name)? {
        if args.args.len() == 1 {
            if let GenericArgument::Type(ty) = &args.args[0] {
                return Some(ty);
            }
        }
    }
    None
}
fn is_type(ty: &Type, ns: &[&[&str]], name: &str) -> bool {
    if let Some(a) = get_argumnets_of(ty, ns, name) {
        a.is_empty()
    } else {
        false
    }
}
fn get_argumnets_of<'a>(ty: &'a Type, ns: &[&[&str]], name: &str) -> Option<&'a PathArguments> {
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
fn is_match_ns(ss: &Punctuated<PathSegment, Token![::]>, ns: &[&str]) -> bool {
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

const NS_STRUCTMETA: &[&[&str]] = &[&["structmeta"]];
const NS_PRIMITIVE: &[&[&str]] = &[&["std", "primitive"], &["core", "primitive"]];

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_is_option() {
        assert_eq!(
            get_option_element(&parse_quote!(Option<u8>)),
            Some(&parse_quote!(u8))
        );
    }
    #[test]
    fn test_is_option_mod() {
        assert_eq!(
            get_option_element(&parse_quote!(option::Option<u8>)),
            Some(&parse_quote!(u8))
        );
    }
    #[test]
    fn test_is_option_core() {
        assert_eq!(
            get_option_element(&parse_quote!(core::option::Option<u8>)),
            Some(&parse_quote!(u8))
        );
    }
    #[test]
    fn test_is_option_std() {
        assert_eq!(
            get_option_element(&parse_quote!(std::option::Option<u8>)),
            Some(&parse_quote!(u8))
        );
    }
}
