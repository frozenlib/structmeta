use proc_macro2::{Ident, Span};
use syn::{
    ext::IdentExt,
    parse::{discouraged::Speculative, ParseStream},
    token, Result, Token,
};

pub enum NameIndex {
    Flag(std::result::Result<usize, Ident>),
    NameValue(std::result::Result<usize, Ident>),
    NameArgs(std::result::Result<usize, Ident>),
}

#[allow(clippy::too_many_arguments)]
pub fn try_parse_name(
    input: ParseStream,
    flag_names: &[&str],
    flag_map: bool,
    name_value_names: &[&str],
    name_value_map: bool,
    name_args_names: &[&str],
    name_args_map: bool,
    no_unnamed: bool,
) -> Result<Option<(NameIndex, Span)>> {
    let fork = input.fork();
    let ident = Ident::parse_any(&fork);
    if no_unnamed {
        ident.clone()?;
    }
    if let Ok(ident) = &ident {
        let span = ident.span();
        let mut is_named = false;
        if (flag_map || !flag_names.is_empty()) && (fork.peek(Token![,]) || fork.is_empty()) {
            if let Some(i) = name_index_of(flag_names, flag_map, &ident) {
                input.advance_to(&fork);
                return Ok(Some((NameIndex::Flag(i), span)));
            }
            is_named = true;
        }
        if (name_value_map || !name_value_names.is_empty()) && fork.peek(Token![=]) {
            if let Some(i) = name_index_of(name_value_names, name_value_map, &ident) {
                fork.parse::<Token![=]>()?;
                input.advance_to(&fork);
                return Ok(Some((NameIndex::NameValue(i), span)));
            }
            is_named = true;
        }
        if (name_args_map || !name_args_names.is_empty()) && fork.peek(token::Paren) {
            if let Some(i) = name_index_of(name_args_names, name_args_map, &ident) {
                input.advance_to(&fork);
                return Ok(Some((NameIndex::NameArgs(i), span)));
            }
            is_named = true;
        }
        if is_named {
            let mut expected = Vec::new();
            if let Some(i) = find(flag_names, &ident) {
                expected.push(format!("flag parameter `{}`", flag_names[i]));
            }
            if let Some(i) = find(name_value_names, &ident) {
                expected.push(format!(
                    "name value parameter `{} = ...`",
                    name_value_names[i]
                ));
            }
            if let Some(i) = find(name_args_names, &ident) {
                expected.push(format!("name args parameter `{}(...)`", name_args_names[i]));
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
fn name_index_of(
    names: &[&str],
    map: bool,
    ident: &Ident,
) -> Option<std::result::Result<usize, Ident>> {
    if let Some(index) = find(names, ident) {
        Some(Ok(index))
    } else if map {
        Some(Err(ident.clone()))
    } else {
        None
    }
}
fn find(names: &[&str], ident: &Ident) -> Option<usize> {
    names.iter().position(|name| ident == name)
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
    m.push('.');
    m
}
