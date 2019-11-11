use crate::data::base::Block;
use crate::data::dict::Dictionary;
use crate::prelude::*;
use derive_new::new;
use indexmap::IndexMap;
use std::io::Write;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Column {
    String(String),
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Shape {
    Primitive(&'static str),
    Row(Vec<Column>),
    Table { from: usize, to: usize },
    Error(ShellError),
    Block(Block),
}

impl Shape {
    pub fn for_value(value: &Value) -> Shape {
        match value {
            Value::Primitive(p) => Shape::Primitive(p.type_name()),
            Value::Row(row) => Shape::for_dict(row),
            Value::Table(table) => Shape::Table {
                from: 0,
                to: table.len(),
            },
            Value::Error(error) => Shape::Error(error.clone()),
            Value::Block(block) => Shape::Block(block.clone()),
        }
    }

    fn for_dict(dict: &Dictionary) -> Shape {
        Shape::Row(dict.keys().map(|key| Column::String(key.clone())).collect())
    }

    pub fn describe(&self, w: &mut impl Write) -> Result<(), std::io::Error> {
        match self {
            Shape::Primitive(desc) => write!(w, "[{}]", desc),
            Shape::Row(d) => write!(
                w,
                "[row: {}]",
                d.iter()
                    .map(|c| match c {
                        Column::String(s) => s,
                    })
                    .join(", ")
            ),
            Shape::Table { to, .. } => write!(w, "[table: {} rows]", to),
            Shape::Error(_) => write!(w, "[error]"),
            Shape::Block(_) => write!(w, "[block]"),
        }
    }

    fn to_value(&self) -> Value {
        let mut out = vec![];
        self.describe(&mut out)
            .expect("Writing into a Vec can't fail");
        let string = String::from_utf8_lossy(&out);

        Value::string(string)
    }
}

#[derive(new)]
pub struct Shapes {
    #[new(default)]
    shapes: IndexMap<Shape, Vec<usize>>,
}

impl Shapes {
    pub fn add(&mut self, value: &Value, row: usize) {
        let shape = Shape::for_value(value);

        self.shapes
            .entry(shape)
            .and_modify(|indexes| indexes.push(row))
            .or_insert_with(|| vec![row]);
    }

    pub fn to_values(&self) -> Vec<Tagged<Value>> {
        if self.shapes.len() == 1 {
            let shape = self.shapes.keys().nth(0).unwrap();

            vec![dict! {
                "type" => shape.to_value(),
                "rows" => Value::string("all")
            }]
        } else {
            self.shapes
                .iter()
                .map(|(shape, rows)| {
                    let rows = rows.iter().map(|i| i.to_string()).join(", ");

                    dict! {
                        "type" => shape.to_value(),
                        "rows" => Value::string(format!("[ {} ]", rows))
                    }
                })
                .collect()
        }
    }
}
