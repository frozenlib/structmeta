use proc_macro2::{Ident, Span};
use syn::{
    ext::IdentExt,
    parse::{discouraged::Speculative, ParseStream},
    token, Result, Token,
};

pub fn try_parse_name(
    input: ParseStream,
    flag: &[&str],
    name_value: &[&str],
    name_args: &[&str],
    no_unnamed: bool,
) -> Result<Option<(usize, Span)>> {
    let fork = input.fork();
    let ident = Ident::parse_any(&fork);
    if no_unnamed {
        ident.clone()?;
    }
    if let Ok(ident) = &ident {
        let span = ident.span();
        let mut is_named = false;
        let base_index = 0;
        if !flag.is_empty() && (fork.peek(Token![,]) || fork.is_empty()) {
            if let Some(i) = find(flag, &ident) {
                input.advance_to(&fork);
                return Ok(Some((base_index + i, span)));
            }
            is_named = true;
        }
        let base_index = base_index + flag.len();
        if !name_value.is_empty() && fork.peek(Token![=]) {
            if let Some(i) = find(name_value, &ident) {
                fork.parse::<Token![=]>()?;
                input.advance_to(&fork);
                return Ok(Some((base_index + i, span)));
            }
            is_named = true;
        }
        let base_index = base_index + name_value.len();
        if !name_args.is_empty() && fork.peek(token::Paren) {
            if let Some(i) = find(name_args, &ident) {
                input.advance_to(&fork);
                return Ok(Some((base_index + i, span)));
            }
            is_named = true;
        }
        if is_named {
            let mut expected = Vec::new();
            if let Some(i) = find(flag, &ident) {
                expected.push(format!("flag parameter `{}`", flag[i]));
            }
            if let Some(i) = find(name_value, &ident) {
                expected.push(format!("name value parameter `{} = ...`", name_value[i]));
            }
            if let Some(i) = find(name_args, &ident) {
                expected.push(format!("name args parameter `{}(...)`", name_args[i]));
            }
            if !expected.is_empty() {
                return Err(input.error(msg(&expected)));
            }
        }
        if is_named || no_unnamed {
            return Err(input.error(format!("cannot find parameter `{}` in this scope", ident)));
        }
    }
    Ok(None)
}
fn find(names: &[&str], ident: &Ident) -> Option<usize> {
    for i in 0..names.len() {
        if ident == names[i] {
            return Some(i);
        }
    }
    None
}
fn msg(expected: &[String]) -> String {
    if expected.is_empty() {
        return "unexpected token.".into();
    }
    let mut m = String::new();
    m.push_str("expected ");
    for i in 0..expected.len() {
        if i != 0 {
            let sep = if i == expected.len() - 1 {
                " or "
            } else {
                ", "
            };
            m.push_str(sep);
            m.push_str(&expected[i]);
        }
    }
    m.push_str(".");
    m
}
