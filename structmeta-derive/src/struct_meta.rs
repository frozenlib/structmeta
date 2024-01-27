use crate::syn_utils::*;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use std::collections::BTreeMap;
use syn::{
    ext::IdentExt,
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    Attribute, Data, DeriveInput, Field, Fields, GenericArgument, Ident, LitStr, PathArguments,
    Result, Token, Type,
};

pub fn derive_struct_meta(input: DeriveInput) -> Result<TokenStream> {
    if let Data::Struct(data) = &input.data {
        let mut args = ArgsForStruct::default();
        for attr in &input.attrs {
            if attr.path().is_ident("struct_meta") {
                args.parse_from_attr(attr)?;
            }
        }
        let ps = Params::from_fields(&data.fields, &args)?;
        let body = ps.build();
        impl_trait_result(
            &input,
            &parse_quote!(::structmeta::helpers::exports::syn::parse::Parse),
            &[],
            quote! {
                fn parse(input: ::structmeta::helpers::exports::syn::parse::ParseStream<'_>) -> ::structmeta::helpers::exports::syn::Result<Self> {
                    #body
                }
            },
            args.dump,
        )
    } else {
        let span = input.span();
        bail!(span, "`#[derive(StructMeta)]` supports only struct.")
    }
}
struct Params<'a> {
    fields: &'a Fields,
    unnamed_required: Vec<UnnamedParam<'a>>,
    unnamed_optional: Vec<UnnamedParam<'a>>,
    unnamed_variadic: Option<UnnamedParam<'a>>,
    named: BTreeMap<String, NamedParam<'a>>,
    rest: Option<RestParam<'a>>,
    name_filter: NameFilter,
}
impl<'a> Params<'a> {
    fn from_fields(fields: &'a Fields, args: &ArgsForStruct) -> Result<Self> {
        let mut unnamed_required = Vec::new();
        let mut unnamed_optional = Vec::new();
        let mut unnamed_variadic = None;
        let mut named = BTreeMap::new();
        let mut rest = None;
        for (index, field) in fields.iter().enumerate() {
            let span = field.span();
            match Param::from_field(index, field)? {
                Param::Unnamed(p) => {
                    if unnamed_variadic.is_some() {
                        bail!(
                            span,
                            "cannot use unnamed parameter after variadic parameter."
                        )
                    }
                    if p.is_vec {
                        unnamed_variadic = Some(p);
                    } else if p.is_option {
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
                Param::Named(p) => {
                    if named.contains_key(&p.name) {
                        bail!(p.name_span, "`{}` is already exists.", p.name);
                    }
                    named.insert(p.name.clone(), p);
                }
                Param::Rest(p) => {
                    if rest.is_some() {
                        bail!(span, "cannot use rest parameter twice.")
                    }
                    rest = Some(p);
                }
            }
        }
        Ok(Self {
            fields,
            unnamed_required,
            unnamed_optional,
            unnamed_variadic,
            named,
            rest,
            name_filter: args.name_filter(),
        })
    }
    fn build(&self) -> TokenStream {
        let mut is_next = false;
        let mut ts = TokenStream::new();
        let mut ctor_args = vec![TokenStream::new(); self.fields.len()];
        for (index, p) in self.unnamed_required.iter().enumerate() {
            if is_next {
                let msg = format!(
                    "expected least {} arguments but {} argument was supplied",
                    self.unnamed_required.len(),
                    index,
                );
                ts.extend(quote! {
                    if input.is_empty () {
                        return Err(::structmeta::helpers::exports::syn::Error::new(input.span(), #msg));
                    }
                    input.parse::<::structmeta::helpers::exports::syn::Token![,]>()?;
                });
            }
            is_next = true;
            ts.extend(p.info.build_let_parse());
            p.build_ctor_arg(false, &mut ctor_args);
        }

        let mut arms_unnamed = Vec::new();
        for (index, p) in self.unnamed_optional.iter().enumerate() {
            ts.extend(p.info.build_let_none());
            arms_unnamed.push(p.build_arm_parse_value(index));
            p.build_ctor_arg(true, &mut ctor_args);
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
        let (flag_ps, flag_rest) = self.named_ps(|p| p.is_flag());
        let (name_value_ps, name_value_rest) = self.named_ps(|p| p.is_name_value());
        let (name_args_ps, name_args_rest) = self.named_ps(|p| p.is_name_args());

        let mut arms_named = Vec::new();
        for (index, p) in flag_ps.iter().enumerate() {
            arms_named.push(p.build_arm_parse(index, ArgKind::Flag));
        }
        for (index, p) in name_value_ps.iter().enumerate() {
            arms_named.push(p.build_arm_parse(index, ArgKind::NameValue));
        }
        for (index, p) in name_args_ps.iter().enumerate() {
            arms_named.push(p.build_arm_parse(index, ArgKind::NameArgs));
        }
        if let Some(p) = &self.rest {
            ts.extend(p.build_let());
            p.build_ctor_arg(&mut ctor_args);
            if flag_rest {
                arms_named.push(p.build_arm_parse(ArgKind::Flag));
            }
            if name_value_rest {
                arms_named.push(p.build_arm_parse(ArgKind::NameValue));
            }
            if name_args_rest {
                arms_named.push(p.build_arm_parse(ArgKind::NameArgs));
            }
        }

        let flag_names = NamedParam::names(&flag_ps);
        let name_value_names = NamedParam::names(&name_value_ps);
        let name_args_names = NamedParam::names(&name_args_ps);
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
        let name_filter = self.name_filter.to_code();

        ts.extend(quote! {
            let mut is_next = #is_next;
            let mut unnamed_index = 0;
            let mut named_used = false;
            while !input.is_empty() {
                if is_next {
                    input.parse::<::structmeta::helpers::exports::syn::Token![,]>()?;
                    if input.is_empty() {
                        break;
                    }
                }
                is_next = true;
                if let Some((index, span)) = ::structmeta::helpers::try_parse_name(input,
                    &[#(#flag_names,)*],
                    #flag_rest,
                    &[#(#name_value_names,)*],
                    #name_value_rest,
                    &[#(#name_args_names,)*],
                    #name_args_rest,
                    #no_unnamed,
                    #name_filter)?
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
    fn named_ps(&self, f: impl Fn(&NamedParamType<'a>) -> bool) -> (Vec<&NamedParam<'a>>, bool) {
        (
            self.named.values().filter(|p| f(&p.ty)).collect(),
            if let Some(p) = &self.rest {
                f(&p.ty)
            } else {
                false
            },
        )
    }
}

enum Param<'a> {
    Unnamed(UnnamedParam<'a>),
    Named(NamedParam<'a>),
    Rest(RestParam<'a>),
}

impl<'a> Param<'a> {
    fn from_field(index: usize, field: &'a Field) -> Result<Self> {
        let mut name = None;
        let mut name_specified = false;
        let mut unnamed = false;
        for attr in &field.attrs {
            if attr.path().is_ident("struct_meta") {
                let a = attr.parse_args::<ArgsForField>()?;
                if let Some(a_name) = a.name {
                    name = Some((a_name.value(), a_name.span()));
                    name_specified = true;
                }
                if a.unnamed {
                    unnamed = true;
                }
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

        let mut is_map = false;
        let mut is_option = false;

        let ty = if let (false, Some(ty)) = (name_specified, get_hash_map_string_element(&field.ty))
        {
            is_map = true;
            ty
        } else if let Some(ty) = get_option_element(&field.ty) {
            is_option = true;
            ty
        } else {
            &field.ty
        };

        let info = ParamInfo::new(index, field, ty);
        let ty = NamedParamType::from_type(ty, !is_map && !is_option);
        let this = if is_map {
            Param::Rest(RestParam { info, ty })
        } else if let Some((name, name_span)) = name {
            Param::Named(NamedParam {
                info,
                name,
                name_span,
                ty,
                is_option,
            })
        } else if let NamedParamType::Value { ty, is_vec } = ty {
            Param::Unnamed(UnnamedParam {
                info,
                ty,
                is_option,
                is_vec,
            })
        } else {
            bail!(
                info.span(),
                "this field type cannot be used as unnamed parameter."
            )
        };
        Ok(this)
    }
}

struct ParamInfo<'a> {
    index: usize,
    field: &'a Field,
    ty: &'a Type,
    temp_ident: Ident,
}
impl<'a> ParamInfo<'a> {
    fn new(index: usize, field: &'a Field, ty: &'a Type) -> Self {
        let temp_ident = format_ident!("_value_{}", index);
        Self {
            index,
            field,
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

struct RestParam<'a> {
    info: ParamInfo<'a>,
    ty: NamedParamType<'a>,
}

struct NamedParam<'a> {
    info: ParamInfo<'a>,
    name: String,
    name_span: Span,
    ty: NamedParamType<'a>,
    is_option: bool,
}

struct UnnamedParam<'a> {
    info: ParamInfo<'a>,
    ty: &'a Type,
    is_option: bool,
    is_vec: bool,
}
impl<'a> NamedParam<'a> {
    fn build_let(&self) -> TokenStream {
        let temp_ident = &self.info.temp_ident;
        quote!(let mut #temp_ident = None;)
    }
    fn build_arm_parse(&self, index: usize, kind: ArgKind) -> TokenStream {
        let temp_ident = &self.info.temp_ident;
        let msg = format!("parameter `{}` specified more than once", self.name);
        let span = self.info.field.span();
        let expr = self.ty.build_parse_expr(kind, span);
        let var = kind.to_helper_name_index_variant();
        quote_spanned! { span=>
            ::structmeta::helpers::NameIndex::#var(Ok(#index)) => {
                if #temp_ident.is_some() {
                    return Err(::structmeta::helpers::exports::syn::Error::new(span, #msg));
                }
                #temp_ident = Some(#expr);
            }
        }
    }
    fn names<'b>(ps: &[&'b Self]) -> Vec<&'b str> {
        ps.iter().map(|x| x.name.as_str()).collect()
    }
    fn build_ctor_arg(&self, ctor_args: &mut [TokenStream]) {
        let temp_ident = &self.info.temp_ident;
        let value = if self.is_option {
            quote!(#temp_ident)
        } else {
            match self.ty {
                NamedParamType::Flag => quote!(::structmeta::Flag { span: #temp_ident }),
                NamedParamType::Bool => quote!(#temp_ident.is_some()),
                NamedParamType::Value { .. } | NamedParamType::NameValue { .. } => {
                    let msg = format!("missing argument `{} = ...`", self.name);
                    quote!(#temp_ident.ok_or_else(|| ::structmeta::helpers::exports::syn::Error::new(::structmeta::helpers::exports::proc_macro2::Span::call_site(), #msg))?)
                }
                NamedParamType::NameArgs { .. } => {
                    let msg = format!("missing argument `{}(...)`", self.name);
                    quote!(#temp_ident.ok_or_else(|| ::structmeta::helpers::exports::syn::Error::new(::structmeta::helpers::exports::proc_macro2::Span::call_site(), #msg))?)
                }
            }
        };
        build_ctor_arg(&self.info, value, ctor_args)
    }
}
impl<'a> RestParam<'a> {
    fn build_let(&self) -> TokenStream {
        let temp_ident = &self.info.temp_ident;
        quote!(let mut #temp_ident = ::std::collections::HashMap::new();)
    }
    fn build_arm_parse(&self, kind: ArgKind) -> TokenStream {
        let temp_ident = &self.info.temp_ident;
        let span = self.info.field.span();
        let expr = self.ty.build_parse_expr(kind, span);
        let var = kind.to_helper_name_index_variant();
        quote_spanned! { span=>
            ::structmeta::helpers::NameIndex::#var(Err(name)) => {
                if #temp_ident.insert(name.to_string(), #expr).is_some() {
                    return Err(::structmeta::helpers::exports::syn::Error::new(span, format!("parameter `{}` specified more than once", name)));
                }
            }
        }
    }
    fn build_ctor_arg(&self, ctor_args: &mut [TokenStream]) {
        let temp_ident = &self.info.temp_ident;
        build_ctor_arg(&self.info, quote!(#temp_ident), ctor_args)
    }
}
impl<'a> UnnamedParam<'a> {
    fn build_arm_parse_value(&self, index: usize) -> TokenStream {
        let temp_ident = &self.info.temp_ident;
        let span = self.info.field.span();
        let expr = build_parse_expr(self.ty, span);
        quote_spanned! { span=>
            #index => {
                #temp_ident = Some(#expr);
            }
        }
    }
    fn build_arm_parse_vec_item(&self) -> TokenStream {
        let temp_ident = &self.info.temp_ident;
        let span = self.info.field.span();
        let expr = build_parse_expr(self.ty, span);
        quote_spanned! { self.info.field.span()=>
            _ => {
                #temp_ident.push(#expr);
            }
        }
    }
    fn build_ctor_arg(&self, var_is_option: bool, ctor_args: &mut [TokenStream]) {
        let temp_ident = &self.info.temp_ident;
        let value = match (var_is_option, self.is_option) {
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
    custom_keyword!(name_filter);
    custom_keyword!(name);
    custom_keyword!(unnamed);
}

#[derive(Debug, Clone, Copy)]
enum NameFilter {
    None,
    SnakeCase,
}
impl NameFilter {
    fn to_code(self) -> TokenStream {
        match self {
            NameFilter::None => quote!(&|_| true),
            NameFilter::SnakeCase => quote!(&::structmeta::helpers::is_snake_case),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct ArgsForStruct {
    dump: bool,
    name_filter: Option<NameFilter>,
}
impl ArgsForStruct {
    fn parse_from_attr(&mut self, attr: &Attribute) -> Result<()> {
        let args = attr.parse_args_with(Punctuated::<ArgForStruct, Token![,]>::parse_terminated)?;
        for arg in args.into_iter() {
            match arg {
                ArgForStruct::Dump(_) => self.dump = true,
                ArgForStruct::NameFilter { span, value } => {
                    if self.name_filter.is_some() {
                        bail!(span, "`name_filter` cannot be specified twice");
                    }
                    self.name_filter = Some(value);
                }
            }
        }
        Ok(())
    }
    fn name_filter(&self) -> NameFilter {
        self.name_filter.unwrap_or(NameFilter::None)
    }
}

enum ArgForStruct {
    Dump(kw::dump),
    NameFilter { span: Span, value: NameFilter },
}
impl Parse for ArgForStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(kw::dump) {
            return Ok(Self::Dump(input.parse()?));
        }
        if input.peek(kw::name_filter) {
            let kw_name_filter: kw::name_filter = input.parse()?;
            let _eq: Token![=] = input.parse()?;
            let s: LitStr = input.parse()?;
            let value = match s.value().as_str() {
                "snake_case" => NameFilter::SnakeCase,
                _ => {
                    bail!(s.span(), "expected \"snake_case\"")
                }
            };
            return Ok(Self::NameFilter {
                span: kw_name_filter.span,
                value,
            });
        }
        Err(input.error("usage : #[struct_meta(dump)]"))
    }
}

struct ArgsForField {
    name: Option<LitStr>,
    unnamed: bool,
}
impl Parse for ArgsForField {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut name = None;
        let mut unnamed = false;
        for p in Punctuated::<_, Token![,]>::parse_terminated(input)?.into_iter() {
            match p {
                ArgForField::Name { value, .. } => name = Some(value),
                ArgForField::Unnamed { .. } => unnamed = true,
            }
        }
        Ok(Self { name, unnamed })
    }
}

enum ArgForField {
    Name {
        _name_token: kw::name,
        _eq_token: Token![=],
        value: LitStr,
    },
    Unnamed {
        _unnamed_token: kw::unnamed,
    },
}
impl Parse for ArgForField {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(kw::name) && input.peek2(Token![=]) {
            let name_token = input.parse()?;
            let eq_token = input.parse()?;
            let value = input.parse()?;
            Ok(Self::Name {
                _name_token: name_token,
                _eq_token: eq_token,
                value,
            })
        } else if input.peek(kw::unnamed) {
            Ok(Self::Unnamed {
                _unnamed_token: input.parse()?,
            })
        } else {
            Err(input.error("expected `name = \"...\"` or `unnamed`."))
        }
    }
}

enum NamedParamType<'a> {
    Bool,
    Flag,
    Value {
        ty: &'a Type,
        is_vec: bool,
    },
    NameValue {
        ty: &'a Type,
        is_option: bool,
    },
    NameArgs {
        ty: &'a Type,
        is_option: bool,
        is_vec: bool,
    },
}

impl<'a> NamedParamType<'a> {
    fn from_type(ty: &'a Type, may_flag: bool) -> Self {
        if may_flag && is_bool(ty) {
            Self::Bool
        } else if may_flag && is_flag(ty) {
            Self::Flag
        } else if let Some(mut ty) = get_name_value_element(ty) {
            let mut is_option = false;
            if let Some(e) = get_option_element(ty) {
                is_option = true;
                ty = e;
            }
            Self::NameValue { ty, is_option }
        } else if let Some(mut ty) = get_name_args_element(ty) {
            let mut is_option = false;
            if let Some(e) = get_option_element(ty) {
                is_option = true;
                ty = e;
            }
            let mut is_vec = false;
            if let Some(e) = get_vec_element(ty) {
                is_vec = true;
                ty = e;
            }
            Self::NameArgs {
                ty,
                is_option,
                is_vec,
            }
        } else {
            let mut ty = ty;
            let mut is_vec = false;
            if let Some(e) = get_vec_element(ty) {
                is_vec = true;
                ty = e;
            }
            Self::Value { ty, is_vec }
        }
    }
    fn is_flag(&self) -> bool {
        match self {
            NamedParamType::Bool | NamedParamType::Flag => true,
            NamedParamType::Value { .. } => false,
            NamedParamType::NameValue { is_option, .. }
            | NamedParamType::NameArgs { is_option, .. } => *is_option,
        }
    }
    fn is_name_value(&self) -> bool {
        match self {
            NamedParamType::Bool | NamedParamType::Flag => false,
            NamedParamType::Value { is_vec, .. } => !is_vec,
            NamedParamType::NameValue { .. } => true,
            NamedParamType::NameArgs { .. } => false,
        }
    }
    fn is_name_args(&self) -> bool {
        match self {
            NamedParamType::Bool | NamedParamType::Flag => false,
            NamedParamType::Value { is_vec, .. } => *is_vec,
            NamedParamType::NameValue { .. } => false,
            NamedParamType::NameArgs { .. } => true,
        }
    }
    fn build_parse_expr(&self, kind: ArgKind, span: Span) -> TokenStream {
        match self {
            NamedParamType::Bool | NamedParamType::Flag => quote!(span),
            NamedParamType::Value { ty, is_vec } => {
                if *is_vec {
                    build_parse_expr_name_args(ty, *is_vec, span)
                } else {
                    build_parse_expr(ty, span)
                }
            }
            NamedParamType::NameValue { ty, is_option } => {
                let value = if kind == ArgKind::Flag && *is_option {
                    quote!(None)
                } else {
                    let value = build_parse_expr(ty, span);
                    if *is_option {
                        quote!(Some(#value))
                    } else {
                        value
                    }
                };
                quote!(::structmeta::NameValue { name_span : span, value: #value })
            }
            NamedParamType::NameArgs {
                ty,
                is_option,
                is_vec,
            } => {
                let args = if kind == ArgKind::Flag && *is_option {
                    quote!(None)
                } else {
                    let args = build_parse_expr_name_args(ty, *is_vec, span);
                    if *is_option {
                        quote!(Some(#args))
                    } else {
                        args
                    }
                };
                quote!(structmeta::NameArgs { name_span : span, args: #args })
            }
        }
    }
}

fn build_parse_expr(ty: &Type, span: Span) -> TokenStream {
    quote_spanned!(span=> input.parse::<#ty>()?)
}
fn build_parse_expr_name_args(ty: &Type, is_vec: bool, span: Span) -> TokenStream {
    let value = if is_vec {
        quote_spanned!(span=> ::structmeta::helpers::exports::syn::punctuated::Punctuated::<#ty, ::structmeta::helpers::exports::syn::Token![,]>::parse_terminated(&content)?.into_iter().collect())
    } else {
        quote_spanned!(span=> content.parse::<#ty>()?)
    };
    quote! {
        {
            let content;
            ::structmeta::helpers::exports::syn::parenthesized!(content in input);
            #value
        }
    }
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
enum ArgKind {
    Flag,
    NameValue,
    NameArgs,
}
impl ArgKind {
    fn to_helper_name_index_variant(self) -> TokenStream {
        match self {
            Self::Flag => quote!(Flag),
            Self::NameValue => quote!(NameValue),
            Self::NameArgs => quote!(NameArgs),
        }
    }
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
fn get_hash_map_element(ty: &Type) -> Option<(&Type, &Type)> {
    get_element2(
        ty,
        &[&["std", "collections"], &["std", "collections", "hash_map"]],
        "HashMap",
    )
}
fn get_hash_map_string_element(ty: &Type) -> Option<&Type> {
    let (ty_key, ty_value) = get_hash_map_element(ty)?;
    if is_string(ty_key) {
        Some(ty_value)
    } else {
        None
    }
}

fn is_bool(ty: &Type) -> bool {
    is_type(ty, NS_PRIMITIVE, "bool")
}
fn is_flag(ty: &Type) -> bool {
    is_type(ty, NS_STRUCTMETA, "Flag")
}
fn is_string(ty: &Type) -> bool {
    is_type(ty, &[&["std", "string"], &["alloc", "string"]], "String")
}

fn get_element<'a>(ty: &'a Type, ns: &[&[&str]], name: &str) -> Option<&'a Type> {
    if let PathArguments::AngleBracketed(args) = get_arguments_of(ty, ns, name)? {
        if args.args.len() == 1 {
            if let GenericArgument::Type(ty) = &args.args[0] {
                return Some(ty);
            }
        }
    }
    None
}
fn get_element2<'a>(ty: &'a Type, ns: &[&[&str]], name: &str) -> Option<(&'a Type, &'a Type)> {
    if let PathArguments::AngleBracketed(args) = get_arguments_of(ty, ns, name)? {
        if args.args.len() == 2 {
            if let (GenericArgument::Type(ty0), GenericArgument::Type(ty1)) =
                (&args.args[0], &args.args[1])
            {
                return Some((ty0, ty1));
            }
        }
    }
    None
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
