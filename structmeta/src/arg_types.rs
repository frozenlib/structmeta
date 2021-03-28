use proc_macro2::Span;
/// `name` style attribute argument.
///
/// See [`#[derive(StructMeta)]`](macro@crate::StructMeta) documentation for details.
#[derive(Clone, Debug, Default)]
pub struct Flag {
    pub span: Option<Span>,
}
impl Flag {
    pub const NONE: Flag = Flag { span: None };
    pub fn value(&self) -> bool {
        self.span.is_some()
    }
}
impl PartialEq for Flag {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}
impl From<bool> for Flag {
    fn from(value: bool) -> Self {
        Self {
            span: if value { Some(Span::call_site()) } else { None },
        }
    }
}

/// `name = value` style attribute argument.
///
/// See [`#[derive(StructMeta)]`](macro@crate::StructMeta) documentation for details.
#[derive(Copy, Clone, Debug)]
pub struct NameValue<T> {
    pub name_span: Span,
    pub value: T,
}
impl<T: PartialEq> PartialEq for NameValue<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

/// `name(value)` style attribute argument.
///
/// See [`#[derive(StructMeta)]`](macro@crate::StructMeta) documentation for details.
#[derive(Copy, Clone, Debug)]
pub struct NameArgs<T> {
    pub name_span: Span,
    pub args: T,
}
impl<T: PartialEq> PartialEq for NameArgs<T> {
    fn eq(&self, other: &Self) -> bool {
        self.args == other.args
    }
}
