use crate::parser::hir::Expression;
use crate::prelude::*;
use derive_new::new;
use getset::{Getters, MutGetters};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum RawPathMember {
    String(String),
    Int(BigInt),
}

pub type PathMember = Spanned<RawPathMember>;

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
    fn fmt_debug(&self, f: &mut DebugFormatter, source: &str) -> fmt::Result {
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
