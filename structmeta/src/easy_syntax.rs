use proc_macro2::{Ident, Span};
use std::{fmt::Display, str::FromStr};
use syn::{
    ext::IdentExt,
    parse::{Parse, ParseStream},
    Error, LitFloat, LitInt, LitStr, Result,
};

pub struct Keyword<T> {
    pub span: Span,
    pub value: T,
}
impl<T: FromStr> Parse for Keyword<T>
where
    T::Err: Display,
{
    fn parse(input: ParseStream) -> Result<Self> {
        let name = Ident::parse_any(input)?;
        let span = name.span();
        match name.to_string().parse() {
            Ok(value) => Ok(Keyword { span, value }),
            Err(e) => Err(Error::new(span, e)),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct LitValue<T> {
    span: Span,
    value: T,
}
impl<T: PartialEq> PartialEq for LitValue<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
impl<T: PartialEq> PartialEq<T> for LitValue<T> {
    fn eq(&self, other: &T) -> bool {
        self.value.eq(other)
    }
}

macro_rules! impl_parse_lit_value_int {
    ($($ty:ty),*) => {
        $(
        impl Parse for LitValue<$ty> {
            fn parse(input: ParseStream) -> Result<Self> {
                let lit = input.parse::<LitInt>()?;
                Ok(Self {
                    span: lit.span(),
                    value: lit.base10_parse()?,
                })
            }
        }
        )*
    };
}

impl_parse_lit_value_int!(u8, u16, u32, u64, u128);
impl_parse_lit_value_int!(i8, i16, i32, i64, i128);

impl Parse for LitValue<String> {
    fn parse(input: ParseStream) -> Result<Self> {
        let lit = input.parse::<LitStr>()?;
        Ok(LitValue {
            span: lit.span(),
            value: lit.value(),
        })
    }
}

impl Parse for LitValue<f32> {
    fn parse(input: ParseStream) -> Result<Self> {
        let lit = input.parse::<LitFloat>()?;
        Ok(LitValue {
            span: lit.span(),
            value: lit.base10_parse()?,
        })
    }
}
impl Parse for LitValue<f64> {
    fn parse(input: ParseStream) -> Result<Self> {
        let lit = input.parse::<LitFloat>()?;
        Ok(LitValue {
            span: lit.span(),
            value: lit.base10_parse()?,
        })
    }
}
