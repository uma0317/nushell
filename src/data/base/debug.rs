use crate::data::base::Primitive;
use crate::prelude::*;
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
            Value::Error(error) => f.say_simple("error"),
            Value::Block(block) => f.say_simple("block"),
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

impl PrettyDebug for Primitive {
    fn pretty_debug<'arena>(
        &self,
        f: &'arena mut DebugFormatter,
        source: &str,
    ) -> DebugDoc<'arena> {
        match self {
            Primitive::Nothing => f.arena().text("nothing"),
            Primitive::Int(_) => (),
            Primitive::Decimal(_) => (),
            Primitive::Bytes(_) => (),
            Primitive::String(_) => (),
            Primitive::ColumnPath(_) => (),
            Primitive::Pattern(_) => (),
            Primitive::Boolean(_) => (),
            Primitive::Date(_) => (),
            Primitive::Path(_) => (),
            Primitive::Binary(_) => (),
            Primitive::BeginningOfStream => (),
            Primitive::EndOfStream => (),
        }
    }
}
