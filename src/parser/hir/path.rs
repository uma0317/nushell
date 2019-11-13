use crate::parser::hir::Expression;
use crate::prelude::*;
use derive_new::new;
use getset::{Getters, MutGetters};
use pretty::{BoxAllocator, DocAllocator};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum RawPathMember {
    String(String),
    Int(BigInt),
}

pub type PathMember = Spanned<RawPathMember>;

impl Into<DebugDocBuilder> for &PathMember {
    fn into(self) -> DebugDocBuilder {
        match &self.item {
            RawPathMember::String(string) => BoxAllocator.text(format!("{:?}", string)),
            RawPathMember::Int(int) => BoxAllocator.text(format!("{}", int)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq, Getters, Clone, new)]
pub struct ColumnPath {
    #[get = "pub"]
    members: Vec<PathMember>,
}

impl ColumnPath {
    pub fn iter(&self) -> impl Iterator<Item = &PathMember> {
        self.members.iter()
    }
}

impl Into<DebugDocBuilder> for &ColumnPath {
    fn into(self) -> DebugDocBuilder {
        let members: Vec<DebugDocBuilder> =
            self.members.iter().map(|member| member.into()).collect();

        BoxAllocator.text("(").append(
            BoxAllocator
                .text("path")
                .annotate(ShellAnnotation::style("equals"))
                .append(BoxAllocator.space())
                .append(BoxAllocator.intersperse(members, BoxAllocator.space()))
                .nest(1)
                .group(),
        )
    }
}

impl FormatDebug for ColumnPath {
    fn fmt_debug(&self, f: &mut DebugFormatter, source: &str) -> fmt::Result {
        self.members.fmt_debug(f, source)
    }
}

impl HasFallibleSpan for ColumnPath {
    fn maybe_span(&self) -> Option<Span> {
        if self.members.len() == 0 {
            None
        } else {
            Some(span_for_spanned_list(self.members.iter().map(|m| m.span)))
        }
    }
}

impl fmt::Display for RawPathMember {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RawPathMember::String(string) => write!(f, "{}", string),
            RawPathMember::Int(int) => write!(f, "{}", int),
        }
    }
}

impl PathMember {
    pub fn string(string: impl Into<String>, span: impl Into<Span>) -> PathMember {
        RawPathMember::String(string.into()).spanned(span.into())
    }

    pub fn int(int: impl Into<BigInt>, span: impl Into<Span>) -> PathMember {
        RawPathMember::Int(int.into()).spanned(span.into())
    }
}

impl FormatDebug for PathMember {
    fn fmt_debug(&self, f: &mut DebugFormatter, _source: &str) -> fmt::Result {
        match &self.item {
            RawPathMember::String(string) => f.say_str("member", &string),
            RawPathMember::Int(int) => f.say_block("member", |f| write!(f, "{}", int)),
        }
    }
}

#[derive(
    Debug,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Getters,
    MutGetters,
    Serialize,
    Deserialize,
    new,
)]
#[get = "pub(crate)"]
pub struct Path {
    head: Expression,
    #[get_mut = "pub(crate)"]
    tail: Vec<PathMember>,
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.head)?;

        for entry in &self.tail {
            write!(f, ".{}", entry.item)?;
        }

        Ok(())
    }
}

impl Path {
    pub(crate) fn parts(self) -> (Expression, Vec<PathMember>) {
        (self.head, self.tail)
    }
}

impl FormatDebug for Path {
    fn fmt_debug(&self, f: &mut DebugFormatter, source: &str) -> fmt::Result {
        write!(f, "{}", self.head.debug(source))?;

        for part in &self.tail {
            write!(f, ".{}", part.item)?;
        }

        Ok(())
    }
}
