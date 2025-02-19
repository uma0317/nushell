use crate::context::CommandRegistry;
use crate::data::base::Block;
use crate::data::value;
use crate::evaluate::operator::apply_operator;
use crate::prelude::*;
use crate::TaggedDictBuilder;
use log::trace;
use nu_errors::{ArgumentError, ShellError};
use nu_parser::hir::{self, Expression, RawExpression};
use nu_protocol::{
    ColumnPath, Evaluate, Primitive, Scope, UnspannedPathMember, UntaggedValue, Value,
};
use nu_source::Text;

pub(crate) fn evaluate_baseline_expr(
    expr: &Expression,
    registry: &CommandRegistry,
    scope: &Scope,
    source: &Text,
) -> Result<Value, ShellError> {
    let tag = Tag {
        span: expr.span,
        anchor: None,
    };
    match &expr.expr {
        RawExpression::Literal(literal) => Ok(evaluate_literal(literal, source)),
        RawExpression::ExternalWord => Err(ShellError::argument_error(
            "Invalid external word".spanned(tag.span),
            ArgumentError::InvalidExternalWord,
        )),
        RawExpression::FilePath(path) => Ok(value::path(path.clone()).into_value(tag)),
        RawExpression::Synthetic(hir::Synthetic::String(s)) => {
            Ok(value::string(s).into_untagged_value())
        }
        RawExpression::Variable(var) => evaluate_reference(var, scope, source, tag),
        RawExpression::Command(_) => evaluate_command(tag, scope, source),
        RawExpression::ExternalCommand(external) => evaluate_external(external, scope, source),
        RawExpression::Binary(binary) => {
            let left = evaluate_baseline_expr(binary.left(), registry, scope, source)?;
            let right = evaluate_baseline_expr(binary.right(), registry, scope, source)?;

            trace!("left={:?} right={:?}", left.value, right.value);

            match apply_operator(binary.op(), &left, &right) {
                Ok(result) => Ok(result.into_value(tag)),
                Err((left_type, right_type)) => Err(ShellError::coerce_error(
                    left_type.spanned(binary.left().span),
                    right_type.spanned(binary.right().span),
                )),
            }
        }
        RawExpression::List(list) => {
            let mut exprs = vec![];

            for expr in list {
                let expr = evaluate_baseline_expr(expr, registry, scope, source)?;
                exprs.push(expr);
            }

            Ok(UntaggedValue::Table(exprs).into_value(tag))
        }
        RawExpression::Block(block) => Ok(UntaggedValue::Block(Evaluate::new(Block::new(
            block.clone(),
            source.clone(),
            tag.clone(),
        )))
        .into_value(&tag)),
        RawExpression::Path(path) => {
            let value = evaluate_baseline_expr(path.head(), registry, scope, source)?;
            let mut item = value;

            for member in path.tail() {
                let next = item.get_data_by_member(member);

                match next {
                    Err(err) => {
                        let possibilities = item.data_descriptors();

                        if let UnspannedPathMember::String(name) = &member.unspanned {
                            let mut possible_matches: Vec<_> = possibilities
                                .iter()
                                .map(|x| (natural::distance::levenshtein_distance(x, &name), x))
                                .collect();

                            possible_matches.sort();

                            if possible_matches.len() > 0 {
                                return Err(ShellError::labeled_error(
                                    "Unknown column",
                                    format!("did you mean '{}'?", possible_matches[0].1),
                                    &tag,
                                ));
                            } else {
                                return Err(err);
                            }
                        }
                    }
                    Ok(next) => {
                        item = next.clone().value.into_value(&tag);
                    }
                };
            }

            Ok(item.value.clone().into_value(tag))
        }
        RawExpression::Boolean(_boolean) => unimplemented!(),
    }
}

fn evaluate_literal(literal: &hir::Literal, source: &Text) -> Value {
    match &literal.literal {
        hir::RawLiteral::ColumnPath(path) => {
            let members = path
                .iter()
                .map(|member| member.to_path_member(source))
                .collect();

            UntaggedValue::Primitive(Primitive::ColumnPath(ColumnPath::new(members)))
                .into_value(&literal.span)
        }
        hir::RawLiteral::Number(int) => value::number(int.clone()).into_value(literal.span),
        hir::RawLiteral::Size(int, unit) => unit.compute(&int).into_value(literal.span),
        hir::RawLiteral::String(tag) => value::string(tag.slice(source)).into_value(literal.span),
        hir::RawLiteral::GlobPattern(pattern) => value::pattern(pattern).into_value(literal.span),
        hir::RawLiteral::Bare => value::string(literal.span.slice(source)).into_value(literal.span),
    }
}

fn evaluate_reference(
    name: &hir::Variable,
    scope: &Scope,
    source: &Text,
    tag: Tag,
) -> Result<Value, ShellError> {
    trace!("Evaluating {:?} with Scope {:?}", name, scope);
    match name {
        hir::Variable::It(_) => Ok(scope.it.value.clone().into_value(tag)),
        hir::Variable::Other(inner) => match inner.slice(source) {
            x if x == "nu:env" => {
                let mut dict = TaggedDictBuilder::new(&tag);
                for v in std::env::vars() {
                    if v.0 != "PATH" && v.0 != "Path" {
                        dict.insert_untagged(v.0, value::string(v.1));
                    }
                }
                Ok(dict.into_value())
            }
            x if x == "nu:config" => {
                let config = crate::data::config::read(tag.clone(), &None)?;
                Ok(value::row(config).into_value(tag))
            }
            x if x == "nu:path" => {
                let mut table = vec![];
                match std::env::var_os("PATH") {
                    Some(paths) => {
                        for path in std::env::split_paths(&paths) {
                            table.push(value::path(path).into_value(&tag));
                        }
                    }
                    _ => {}
                }
                Ok(value::table(&table).into_value(tag))
            }
            x => Ok(scope
                .vars
                .get(x)
                .map(|v| v.clone())
                .unwrap_or_else(|| value::nothing().into_value(tag))),
        },
    }
}

fn evaluate_external(
    external: &hir::ExternalCommand,
    _scope: &Scope,
    _source: &Text,
) -> Result<Value, ShellError> {
    Err(ShellError::syntax_error(
        "Unexpected external command".spanned(*external.name()),
    ))
}

fn evaluate_command(tag: Tag, _scope: &Scope, _source: &Text) -> Result<Value, ShellError> {
    Err(ShellError::syntax_error(
        "Unexpected command".spanned(tag.span),
    ))
}
