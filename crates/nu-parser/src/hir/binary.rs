use crate::{hir::Expression, Operator};

use derive_new::new;
use getset::Getters;
use nu_source::{b, DebugDocBuilder, PrettyDebugWithSource, Spanned};
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Getters, Serialize, Deserialize, new,
)]
#[get = "pub"]
pub struct Binary {
    left: Expression,
    op: Spanned<Operator>,
    right: Expression,
}

impl PrettyDebugWithSource for Binary {
    fn pretty_debug(&self, source: &str) -> DebugDocBuilder {
        b::delimit(
            "<",
            self.left.pretty_debug(source)
                + b::space()
                + b::keyword(self.op.span.slice(source))
                + b::space()
                + self.right.pretty_debug(source),
            ">",
        )
        .group()
    }
}
