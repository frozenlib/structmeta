use proc_macro2::{Ident, Spacing, Span, TokenStream};
use syn::{
    braced, bracketed,
    ext::IdentExt,
    parenthesized,
    parse::{discouraged::Speculative, ParseBuffer, ParseStream},
    token::{self},
    MacroDelimiter, Result, Token,
};

pub mod exports {
    pub use proc_macro2;
    pub use quote;
    pub use syn;
}

pub enum NameIndex {
    Flag(std::result::Result<usize, Ident>),
    NameValue(std::result::Result<usize, Ident>),
    NameArgs(std::result::Result<usize, Ident>),
}

#[allow(clippy::too_many_arguments)]
pub fn try_parse_name(
    input: ParseStream,
    flag_names: &[&str],
    flag_rest: bool,
    name_value_names: &[&str],
    name_value_rest: bool,
    name_args_names: &[&str],
    name_args_rest: bool,
    no_unnamed: bool,
    name_filter: &dyn Fn(&str) -> bool,
) -> Result<Option<(NameIndex, Span)>> {
    let may_flag = !flag_names.is_empty() || flag_rest;
    let may_name_value = !name_value_names.is_empty() || name_value_rest;
    let may_name_args = !name_args_names.is_empty() || name_args_rest;
    let fork = input.fork();
    if let Ok(ident) = Ident::parse_any(&fork) {
        if name_filter(&ident.to_string()) {
            let span = ident.span();
            let mut kind = None;
            if (no_unnamed || may_flag) && (fork.peek(Token![,]) || fork.is_empty()) {
                if let Some(i) = name_index_of(flag_names, flag_rest, &ident) {
                    input.advance_to(&fork);
                    return Ok(Some((NameIndex::Flag(i), span)));
                }
                kind = Some(ArgKind::Flag);
            } else if (no_unnamed || may_name_value) && peek_eq_op(&fork) {
                if let Some(i) = name_index_of(name_value_names, name_value_rest, &ident) {
                    fork.parse::<Token![=]>()?;
                    input.advance_to(&fork);
                    return Ok(Some((NameIndex::NameValue(i), span)));
                }
                kind = Some(ArgKind::NameValue);
            } else if (no_unnamed || may_name_args) && fork.peek(token::Paren) {
                if let Some(i) = name_index_of(name_args_names, name_args_rest, &ident) {
                    input.advance_to(&fork);
                    return Ok(Some((NameIndex::NameArgs(i), span)));
                }
                kind = Some(ArgKind::NameArgs);
            };

            if kind.is_some() || no_unnamed {
                let mut expected = Vec::new();
                if let Some(name) = name_of(flag_names, flag_rest, &ident) {
                    expected.push(format!("flag `{name}`"));
                }
                if let Some(name) = name_of(name_value_names, name_value_rest, &ident) {
                    expected.push(format!("`{name} = ...`"));
                }
                if let Some(name) = name_of(name_args_names, name_args_rest, &ident) {
                    expected.push(format!("`{name}(...)`"));
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
                let help = if let Some(similar_name) =
                    find_similar_name(&[flag_names, name_value_names, name_args_names], &ident)
                {
                    format!(" (help: a parameter with a similar name exists: `{similar_name}`)",)
                } else {
                    "".into()
                };
                return Err(input.error(format!(
                    "cannot find parameter `{ident}` in this scope{help}"
                )));
            }
        }
    }
    if no_unnamed {
        let message = if may_flag || may_name_value || may_name_args {
            "too many unnamed arguments."
        } else {
            "too many arguments."
        };
        return Err(input.error(message));
    }
    Ok(None)
}
fn peek_eq_op(input: ParseStream) -> bool {
    if let Some((p, _)) = input.cursor().punct() {
        p.as_char() == '=' && p.spacing() == Spacing::Alone
    } else {
        false
    }
}
fn name_index_of(
    names: &[&str],
    rest: bool,
    ident: &Ident,
) -> Option<std::result::Result<usize, Ident>> {
    if let Some(index) = find(names, ident) {
        Some(Ok(index))
    } else if rest {
        Some(Err(ident.clone()))
    } else {
        None
    }
}
fn name_of(names: &[&str], rest: bool, ident: &Ident) -> Option<String> {
    if rest {
        Some(ident.to_string())
    } else {
        find(names, ident).map(|i| names[i].to_string())
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
fn find_similar_name<'a>(names: &[&[&'a str]], ident: &Ident) -> Option<&'a str> {
    let c0: Vec<_> = ident.to_string().chars().collect();
    let mut c1 = Vec::new();
    let mut r = None;
    let mut r_d = usize::max_value();
    for &names in names {
        for &name in names {
            c1.clear();
            c1.extend(name.chars());
            if let Some(d) = distance(&c0, &c1) {
                if d < r_d {
                    r_d = d;
                    r = Some(name);
                }
                if d == r_d && Some(name) != r {
                    return None;
                }
            }
        }
    }
    r
}

fn distance(s0: &[char], s1: &[char]) -> Option<usize> {
    if s0.len() > s1.len() {
        return distance(s1, s0);
    }
    if s0.len() + 1 < s1.len() {
        return None;
    }
    let mut start = 0;
    while start < s0.len() && start < s1.len() && s0[start] == s1[start] {
        start += 1;
    }
    let mut end = 0;
    while start + end < s0.len()
        && start + end < s1.len()
        && s0[s0.len() - end - 1] == s1[s1.len() - end - 1]
    {
        end += 1;
    }
    if s0.len() == s1.len() {
        if start + end == s0.len() {
            return Some(0);
        }
        if start + end + 1 == s0.len() {
            return Some(1);
        }
        if start + end + 2 == s0.len() && s0[start] == s1[start + 1] && s0[start + 1] == s1[start] {
            return Some(2);
        }
    } else if start + end == s0.len() {
        return Some(1);
    }

    None
}

pub fn is_snake_case(s: &str) -> bool {
    s.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

pub fn surround_macro_delimiter<F>(this: &MacroDelimiter, tokens: &mut TokenStream, f: F)
where
    F: FnOnce(&mut TokenStream),
{
    match this {
        MacroDelimiter::Paren(p) => p.surround(tokens, f),
        MacroDelimiter::Bracket(b) => b.surround(tokens, f),
        MacroDelimiter::Brace(b) => b.surround(tokens, f),
    }
}

pub fn parse_macro_delimiter<'a>(
    input: &ParseBuffer<'a>,
) -> Result<(MacroDelimiter, ParseBuffer<'a>)> {
    let content;
    let token = if input.peek(token::Paren) {
        MacroDelimiter::Paren(parenthesized!(content in input))
    } else if input.peek(token::Bracket) {
        MacroDelimiter::Bracket(bracketed!(content in input))
    } else if input.peek(token::Brace) {
        MacroDelimiter::Brace(braced!(content in input))
    } else {
        return Err(input.error("expected `(`, `[` or `{`"));
    };
    Ok((token, content))
}

#[doc(hidden)]
#[macro_export]
macro_rules! helpers_parse_macro_delimiter {
    ($content:ident in $input:ident) => {
        match $crate::helpers::parse_macro_delimiter($input) {
            Ok((token, content)) => {
                $content = content;
                token
            }
            Err(e) => return Err(e),
        }
    };
}

#[test]
fn test_is_near() {
    fn check(s0: &str, s1: &str, e: Option<usize>) {
        let c0: Vec<_> = s0.chars().collect();
        let c1: Vec<_> = s1.chars().collect();
        assert_eq!(distance(&c0, &c1), e, "{s0} , {s1}")
    }
    check("a", "a", Some(0));
    check("a", "b", Some(1));
    check("a", "ab", Some(1));
    check("ab", "a", Some(1));
    check("a", "aa", Some(1));
    check("ab", "ba", Some(2));
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
