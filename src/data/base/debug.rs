use crate::data::base::Primitive;
use crate::prelude::*;
use pretty::{BoxAllocator, DocAllocator};
use std::fmt;

impl FormatDebug for Tagged<Value> {
    fn fmt_debug(&self, f: &mut DebugFormatter, source: &str) -> fmt::Result {
        match &self.item {
            Value::Primitive(p) => p.fmt_debug(f, source),
            Value::Row(row) => f.say_dict(
                "row",
                row.entries()
                    .iter()
                    .map(|(key, value)| (&key[..], format!("{}", value.debug(source))))
                    .collect(),
            ),
            Value::Table(table) => f.say_list(
                "table",
                table,
                |f| write!(f, "["),
                |f, item| write!(f, "{}", item.debug(source)),
                |f| write!(f, " "),
                |f| write!(f, "]"),
            ),
            Value::Error(_) => f.say_simple("error"),
            Value::Block(_) => f.say_simple("block"),
        }
    }
}

impl FormatDebug for Primitive {
    fn fmt_debug(&self, f: &mut DebugFormatter, source: &str) -> fmt::Result {
        use Primitive::*;

        match self {
            Nothing => write!(f, "Nothing"),
            BeginningOfStream => write!(f, "BeginningOfStream"),
            EndOfStream => write!(f, "EndOfStream"),
            Int(int) => write!(f, "{}", int),
            Path(path) => write!(f, "{}", path.display()),
            Decimal(decimal) => write!(f, "{}", decimal),
            Bytes(bytes) => write!(f, "{}", bytes),
            Pattern(string) => write!(f, "{:?}", string),
            String(string) => write!(f, "{:?}", string),
            ColumnPath(path) => write!(f, "{:?}", path),
            Boolean(boolean) => write!(f, "{}", boolean),
            Date(date) => write!(f, "{}", date),
            Binary(binary) => write!(f, "{:?}", binary),
        }
    }
}

impl From<&Primitive> for DebugDocBuilder {
    fn from(primitive: &Primitive) -> DebugDocBuilder {
        match primitive {
            Primitive::Nothing => BoxAllocator.text("nothing"),
            Primitive::Int(int) => prim(format_args!("{}", int)),
            Primitive::Decimal(decimal) => prim(format_args!("{}", decimal)),
            Primitive::Bytes(bytes) => primitive_doc(bytes, "bytesize"),
            Primitive::String(string) => prim(string),
            Primitive::ColumnPath(path) => path.into(),
            Primitive::Pattern(pattern) => primitive_doc(pattern, "pattern"),
            Primitive::Boolean(boolean) => match boolean {
                true => BoxAllocator
                    .text("$yes")
                    .annotate(ShellAnnotation::style("primitive"))
                    .into(),
                false => BoxAllocator
                    .text("$no")
                    .annotate(ShellAnnotation::style("primitive"))
                    .into(),
            },
            Primitive::Date(date) => primitive_doc(date, "date"),
            Primitive::Path(path) => primitive_doc(path, "path"),
            Primitive::Binary(_) => BoxAllocator
                .text("binary")
                .annotate(ShellAnnotation::style("opaque"))
                .into(),
            Primitive::BeginningOfStream => BoxAllocator
                .text("binary")
                .annotate(ShellAnnotation::style("beginning-of-stream"))
                .into(),
            Primitive::EndOfStream => BoxAllocator
                .text("binary")
                .annotate(ShellAnnotation::style("end-of-stream"))
                .into(),
        }
    }
}

impl From<&Value> for DebugDocBuilder {
    fn from(value: &Value) -> DebugDocBuilder {
        match value {
            Value::Primitive(p) => p.into(),
            Value::Row(row) => DebugDocBuilder::from(row).nest(1).group().into(),
            Value::Table(table) => BoxAllocator
                .text("[")
                .append(
                    BoxAllocator
                        .intersperse(
                            table.iter().map(|v| DebugDocBuilder::from(&v.item)),
                            BoxAllocator.space(),
                        )
                        .nest(1)
                        .group(),
                )
                .append(BoxAllocator.text("]"))
                .into(),
            Value::Error(_) => BoxAllocator
                .text("error")
                .annotate(ShellAnnotation::style("error"))
                .into(),
            Value::Block(_) => BoxAllocator
                .text("block")
                .annotate(ShellAnnotation::style("opaque"))
                .into(),
        }
    }
}

fn prim(name: impl std::fmt::Debug) -> DebugDocBuilder {
    BoxAllocator
        .text(format!("{:?}", name))
        .annotate(ShellAnnotation::style("primitive"))
}

fn primitive_doc(name: impl std::fmt::Debug, ty: impl Into<String>) -> DebugDocBuilder {
    BoxAllocator
        .text(format!("{:?}", name))
        .annotate(ShellAnnotation::style("primitive"))
        .append(
            BoxAllocator
                .text("(")
                .append(BoxAllocator.text(ty.into()))
                .append(BoxAllocator.text(")"))
                .annotate(ShellAnnotation::style("kind"))
        )
}
