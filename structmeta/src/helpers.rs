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
    if let Ok(ident) = Ident::parse_any(&fork) {
        let span = ident.span();
        let mut kind = None;
        if fork.peek(Token![,]) || fork.is_empty() {
            if let Some(i) = name_index_of(flag_names, flag_map, &ident) {
                input.advance_to(&fork);
                return Ok(Some((NameIndex::Flag(i), span)));
            }
            kind = Some(ArgKind::Flag);
        } else if fork.peek(Token![=]) {
            if let Some(i) = name_index_of(name_value_names, name_value_map, &ident) {
                fork.parse::<Token![=]>()?;
                input.advance_to(&fork);
                return Ok(Some((NameIndex::NameValue(i), span)));
            }
            kind = Some(ArgKind::NameValue);
        } else if fork.peek(token::Paren) {
            if let Some(i) = name_index_of(name_args_names, name_args_map, &ident) {
                input.advance_to(&fork);
                return Ok(Some((NameIndex::NameArgs(i), span)));
            }
            kind = Some(ArgKind::NameArgs);
        };

        if kind.is_some() || no_unnamed {
            let mut expected = Vec::new();
            if let Some(name) = name_of(flag_names, flag_map, &ident) {
                expected.push(format!("flag `{}`", name));
            }
            if let Some(name) = name_of(name_value_names, name_value_map, &ident) {
                expected.push(format!("`{} = ...`", name));
            }
            if let Some(name) = name_of(name_args_names, name_args_map, &ident) {
                expected.push(format!("`{}(...)`", name));
            }
            if !expected.is_empty() {
                return Err(input.error(msg(
                    &expected,
                    kind.map(|kind| Arg {
                        kind,
                        ident: &ident,
                    }),
                )));
            }
            return Err(input.error(format!("cannot find parameter `{}` in this scope", ident)));
        }
    }
    if no_unnamed {
        let message = if flag_names.is_empty()
            && !flag_map
            && name_value_names.is_empty()
            && !name_value_map
            && name_args_names.is_empty()
            && !name_args_map
        {
            "too many arguments."
        } else {
            "too many unnamed arguments."
        };
        return Err(input.error(message));
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
fn name_of(names: &[&str], rest: bool, ident: &Ident) -> Option<String> {
    if rest {
        Some(ident.to_string())
    } else if let Some(i) = find(names, ident) {
        Some(names[i].to_string())
    } else {
        None
    }
}
fn find(names: &[&str], ident: &Ident) -> Option<usize> {
    names.iter().position(|name| ident == name)
}
fn msg(expected: &[String], found: Option<Arg>) -> String {
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
        }
        m.push_str(&expected[i]);
    }
    if let Some(arg) = found {
        m.push_str(", found ");
        m.push_str(&match arg.kind {
            ArgKind::Flag => format!("`{}`", arg.ident),
            ArgKind::NameValue => format!("`{} = ...`", arg.ident),
            ArgKind::NameArgs => format!("`{}`(...)", arg.ident),
        });
    }
    m
}
enum ArgKind {
    Flag,
    NameValue,
    NameArgs,
}
struct Arg<'a> {
    kind: ArgKind,
    ident: &'a Ident,
}
