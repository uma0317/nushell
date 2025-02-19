use crate::hir::syntax_shape::FlatShape;
use crate::hir::syntax_shape::{
    expand_syntax, expression::expand_file_path, parse_single_node, BarePathShape,
    BarePatternShape, ExpandContext, UnitShape, UnitSyntax,
};
use crate::parse::token_tree::{DelimitedNode, Delimiter, TokenNode};
use crate::parse::tokens::UnspannedToken;
use crate::parse::unit::Unit;
use crate::{
    hir,
    hir::{Expression, RawNumber, TokensIterator},
    parse::flag::{Flag, FlagKind},
};
use nu_errors::{ParseError, ShellError};
use nu_protocol::ShellTypeName;
use nu_source::{b, DebugDocBuilder, HasSpan, PrettyDebugWithSource, Span, Spanned, SpannedItem};
use std::ops::Deref;

#[derive(Debug, Clone)]
pub enum UnspannedAtomicToken<'tokens> {
    Eof {
        span: Span,
    },
    Error {
        error: Spanned<ShellError>,
    },
    Number {
        number: RawNumber,
    },
    Size {
        number: RawNumber,
        unit: Spanned<Unit>,
    },
    String {
        body: Span,
    },
    ItVariable {
        name: Span,
    },
    Variable {
        name: Span,
    },
    ExternalCommand {
        command: Span,
    },
    ExternalWord {
        text: Span,
    },
    GlobPattern {
        pattern: Span,
    },
    Word {
        text: Span,
    },
    #[allow(unused)]
    Dot {
        text: Span,
    },
    SquareDelimited {
        spans: (Span, Span),
        nodes: &'tokens Vec<TokenNode>,
    },
    ShorthandFlag {
        name: Span,
    },
    Operator {
        text: Span,
    },
    Whitespace {
        text: Span,
    },
}

impl<'tokens> UnspannedAtomicToken<'tokens> {
    pub fn into_atomic_token(self, span: impl Into<Span>) -> AtomicToken<'tokens> {
        AtomicToken {
            unspanned: self,
            span: span.into(),
        }
    }
}

impl<'tokens> ShellTypeName for UnspannedAtomicToken<'tokens> {
    fn type_name(&self) -> &'static str {
        match &self {
            UnspannedAtomicToken::Eof { .. } => "eof",
            UnspannedAtomicToken::Error { .. } => "error",
            UnspannedAtomicToken::Operator { .. } => "operator",
            UnspannedAtomicToken::ShorthandFlag { .. } => "shorthand flag",
            UnspannedAtomicToken::Whitespace { .. } => "whitespace",
            UnspannedAtomicToken::Dot { .. } => "dot",
            UnspannedAtomicToken::Number { .. } => "number",
            UnspannedAtomicToken::Size { .. } => "size",
            UnspannedAtomicToken::String { .. } => "string",
            UnspannedAtomicToken::ItVariable { .. } => "$it",
            UnspannedAtomicToken::Variable { .. } => "variable",
            UnspannedAtomicToken::ExternalCommand { .. } => "external command",
            UnspannedAtomicToken::ExternalWord { .. } => "external word",
            UnspannedAtomicToken::GlobPattern { .. } => "file pattern",
            UnspannedAtomicToken::Word { .. } => "word",
            UnspannedAtomicToken::SquareDelimited { .. } => "array literal",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AtomicToken<'tokens> {
    pub unspanned: UnspannedAtomicToken<'tokens>,
    pub span: Span,
}

impl<'tokens> Deref for AtomicToken<'tokens> {
    type Target = UnspannedAtomicToken<'tokens>;

    fn deref(&self) -> &UnspannedAtomicToken<'tokens> {
        &self.unspanned
    }
}

impl<'tokens> AtomicToken<'tokens> {
    pub fn into_hir(
        &self,
        context: &ExpandContext,
        expected: &'static str,
    ) -> Result<hir::Expression, ParseError> {
        Ok(match &self.unspanned {
            UnspannedAtomicToken::Eof { .. } => {
                return Err(ParseError::mismatch(
                    expected,
                    "eof atomic token".spanned(self.span),
                ))
            }
            UnspannedAtomicToken::Error { .. } => {
                return Err(ParseError::mismatch(
                    expected,
                    "eof atomic token".spanned(self.span),
                ))
            }
            UnspannedAtomicToken::Operator { .. } => {
                return Err(ParseError::mismatch(
                    expected,
                    "operator".spanned(self.span),
                ))
            }
            UnspannedAtomicToken::ShorthandFlag { .. } => {
                return Err(ParseError::mismatch(
                    expected,
                    "shorthand flag".spanned(self.span),
                ))
            }
            UnspannedAtomicToken::Whitespace { .. } => {
                return Err(ParseError::mismatch(
                    expected,
                    "whitespace".spanned(self.span),
                ))
            }
            UnspannedAtomicToken::Dot { .. } => {
                return Err(ParseError::mismatch(expected, "dot".spanned(self.span)))
            }
            UnspannedAtomicToken::Number { number } => {
                Expression::number(number.to_number(context.source), self.span)
            }
            UnspannedAtomicToken::Size { number, unit } => {
                Expression::size(number.to_number(context.source), **unit, self.span)
            }
            UnspannedAtomicToken::String { body } => Expression::string(*body, self.span),
            UnspannedAtomicToken::ItVariable { name } => Expression::it_variable(*name, self.span),
            UnspannedAtomicToken::Variable { name } => Expression::variable(*name, self.span),
            UnspannedAtomicToken::ExternalCommand { command } => {
                Expression::external_command(*command, self.span)
            }
            UnspannedAtomicToken::ExternalWord { text } => Expression::string(*text, self.span),
            UnspannedAtomicToken::GlobPattern { pattern } => Expression::pattern(
                expand_file_path(pattern.slice(context.source), context).to_string_lossy(),
                self.span,
            ),
            UnspannedAtomicToken::Word { text } => Expression::string(*text, *text),
            UnspannedAtomicToken::SquareDelimited { .. } => unimplemented!("into_hir"),
        })
    }

    #[cfg(not(coloring_in_tokens))]
    pub fn spanned_type_name(&self) -> Spanned<&'static str> {
        match &self.unspanned {
            UnspannedAtomicToken::Eof { .. } => "eof",
            UnspannedAtomicToken::Error { .. } => "error",
            UnspannedAtomicToken::Operator { .. } => "operator",
            UnspannedAtomicToken::ShorthandFlag { .. } => "shorthand flag",
            UnspannedAtomicToken::Whitespace { .. } => "whitespace",
            UnspannedAtomicToken::Dot { .. } => "dot",
            UnspannedAtomicToken::Number { .. } => "number",
            UnspannedAtomicToken::Size { .. } => "size",
            UnspannedAtomicToken::String { .. } => "string",
            UnspannedAtomicToken::ItVariable { .. } => "$it",
            UnspannedAtomicToken::Variable { .. } => "variable",
            UnspannedAtomicToken::ExternalCommand { .. } => "external command",
            UnspannedAtomicToken::ExternalWord { .. } => "external word",
            UnspannedAtomicToken::GlobPattern { .. } => "file pattern",
            UnspannedAtomicToken::Word { .. } => "word",
            UnspannedAtomicToken::SquareDelimited { .. } => "array literal",
        }
        .spanned(self.span)
    }

    pub(crate) fn color_tokens(&self, shapes: &mut Vec<Spanned<FlatShape>>) {
        match &self.unspanned {
            UnspannedAtomicToken::Eof { .. } => {}
            UnspannedAtomicToken::Error { .. } => {
                return shapes.push(FlatShape::Error.spanned(self.span))
            }
            UnspannedAtomicToken::Operator { .. } => {
                return shapes.push(FlatShape::Operator.spanned(self.span));
            }
            UnspannedAtomicToken::ShorthandFlag { .. } => {
                return shapes.push(FlatShape::ShorthandFlag.spanned(self.span));
            }
            UnspannedAtomicToken::Whitespace { .. } => {
                return shapes.push(FlatShape::Whitespace.spanned(self.span));
            }
            UnspannedAtomicToken::Number {
                number: RawNumber::Decimal(_),
            } => {
                return shapes.push(FlatShape::Decimal.spanned(self.span));
            }
            UnspannedAtomicToken::Number {
                number: RawNumber::Int(_),
            } => {
                return shapes.push(FlatShape::Int.spanned(self.span));
            }
            UnspannedAtomicToken::Size { number, unit } => {
                return shapes.push(
                    FlatShape::Size {
                        number: number.span(),
                        unit: unit.span,
                    }
                    .spanned(self.span),
                );
            }
            UnspannedAtomicToken::String { .. } => {
                return shapes.push(FlatShape::String.spanned(self.span))
            }
            UnspannedAtomicToken::ItVariable { .. } => {
                return shapes.push(FlatShape::ItVariable.spanned(self.span))
            }
            UnspannedAtomicToken::Variable { .. } => {
                return shapes.push(FlatShape::Variable.spanned(self.span))
            }
            UnspannedAtomicToken::ExternalCommand { .. } => {
                return shapes.push(FlatShape::ExternalCommand.spanned(self.span));
            }
            UnspannedAtomicToken::ExternalWord { .. } => {
                return shapes.push(FlatShape::ExternalWord.spanned(self.span))
            }
            UnspannedAtomicToken::GlobPattern { .. } => {
                return shapes.push(FlatShape::GlobPattern.spanned(self.span))
            }
            UnspannedAtomicToken::Word { .. } => {
                return shapes.push(FlatShape::Word.spanned(self.span))
            }
            _ => return shapes.push(FlatShape::Error.spanned(self.span)),
        }
    }
}

impl PrettyDebugWithSource for AtomicToken<'_> {
    fn pretty_debug(&self, source: &str) -> DebugDocBuilder {
        fn atom(value: DebugDocBuilder) -> DebugDocBuilder {
            b::delimit("(", b::kind("atom") + b::space() + value.group(), ")").group()
        }

        fn atom_kind(kind: impl std::fmt::Display, value: DebugDocBuilder) -> DebugDocBuilder {
            b::delimit(
                "(",
                (b::kind("atom") + b::delimit("[", b::kind(kind), "]")).group()
                    + b::space()
                    + value.group(),
                ")",
            )
            .group()
        }

        atom(match &self.unspanned {
            UnspannedAtomicToken::Eof { .. } => b::description("eof"),
            UnspannedAtomicToken::Error { .. } => b::error("error"),
            UnspannedAtomicToken::Number { number } => number.pretty_debug(source),
            UnspannedAtomicToken::Size { number, unit } => {
                number.pretty_debug(source) + b::keyword(unit.span.slice(source))
            }
            UnspannedAtomicToken::String { body } => b::primitive(body.slice(source)),
            UnspannedAtomicToken::ItVariable { .. } | UnspannedAtomicToken::Variable { .. } => {
                b::keyword(self.span.slice(source))
            }
            UnspannedAtomicToken::ExternalCommand { .. } => b::primitive(self.span.slice(source)),
            UnspannedAtomicToken::ExternalWord { text } => {
                atom_kind("external word", b::primitive(text.slice(source)))
            }
            UnspannedAtomicToken::GlobPattern { pattern } => {
                atom_kind("pattern", b::primitive(pattern.slice(source)))
            }
            UnspannedAtomicToken::Word { text } => {
                atom_kind("word", b::primitive(text.slice(source)))
            }
            UnspannedAtomicToken::SquareDelimited { nodes, .. } => b::delimit(
                "[",
                b::intersperse_with_source(nodes.iter(), b::space(), source),
                "]",
            ),
            UnspannedAtomicToken::ShorthandFlag { name } => {
                atom_kind("shorthand flag", b::key(name.slice(source)))
            }
            UnspannedAtomicToken::Dot { .. } => atom(b::kind("dot")),
            UnspannedAtomicToken::Operator { text } => {
                atom_kind("operator", b::keyword(text.slice(source)))
            }
            UnspannedAtomicToken::Whitespace { text } => atom_kind(
                "whitespace",
                b::description(format!("{:?}", text.slice(source))),
            ),
        })
    }
}

#[derive(Debug)]
pub enum WhitespaceHandling {
    #[allow(unused)]
    AllowWhitespace,
    RejectWhitespace,
}

#[derive(Debug)]
pub struct ExpansionRule {
    pub(crate) allow_external_command: bool,
    pub(crate) allow_external_word: bool,
    pub(crate) allow_operator: bool,
    pub(crate) allow_eof: bool,
    pub(crate) treat_size_as_word: bool,
    pub(crate) separate_members: bool,
    pub(crate) commit_errors: bool,
    pub(crate) whitespace: WhitespaceHandling,
}

impl ExpansionRule {
    pub fn new() -> ExpansionRule {
        ExpansionRule {
            allow_external_command: false,
            allow_external_word: false,
            allow_operator: false,
            allow_eof: false,
            treat_size_as_word: false,
            separate_members: false,
            commit_errors: false,
            whitespace: WhitespaceHandling::RejectWhitespace,
        }
    }

    /// The intent of permissive mode is to return an atomic token for every possible
    /// input token. This is important for error-correcting parsing, such as the
    /// syntax highlighter.
    pub fn permissive() -> ExpansionRule {
        ExpansionRule {
            allow_external_command: true,
            allow_external_word: true,
            allow_operator: true,
            allow_eof: true,
            separate_members: false,
            treat_size_as_word: false,
            commit_errors: true,
            whitespace: WhitespaceHandling::AllowWhitespace,
        }
    }

    #[allow(unused)]
    pub fn allow_external_command(mut self) -> ExpansionRule {
        self.allow_external_command = true;
        self
    }

    #[allow(unused)]
    pub fn allow_operator(mut self) -> ExpansionRule {
        self.allow_operator = true;
        self
    }

    #[allow(unused)]
    pub fn no_operator(mut self) -> ExpansionRule {
        self.allow_operator = false;
        self
    }

    #[allow(unused)]
    pub fn no_external_command(mut self) -> ExpansionRule {
        self.allow_external_command = false;
        self
    }

    #[allow(unused)]
    pub fn allow_external_word(mut self) -> ExpansionRule {
        self.allow_external_word = true;
        self
    }

    #[allow(unused)]
    pub fn no_external_word(mut self) -> ExpansionRule {
        self.allow_external_word = false;
        self
    }

    #[allow(unused)]
    pub fn treat_size_as_word(mut self) -> ExpansionRule {
        self.treat_size_as_word = true;
        self
    }

    #[allow(unused)]
    pub fn separate_members(mut self) -> ExpansionRule {
        self.separate_members = true;
        self
    }

    #[allow(unused)]
    pub fn no_separate_members(mut self) -> ExpansionRule {
        self.separate_members = false;
        self
    }

    #[allow(unused)]
    pub fn commit_errors(mut self) -> ExpansionRule {
        self.commit_errors = true;
        self
    }

    #[allow(unused)]
    pub fn allow_whitespace(mut self) -> ExpansionRule {
        self.whitespace = WhitespaceHandling::AllowWhitespace;
        self
    }

    #[allow(unused)]
    pub fn reject_whitespace(mut self) -> ExpansionRule {
        self.whitespace = WhitespaceHandling::RejectWhitespace;
        self
    }
}

pub fn expand_atom<'me, 'content>(
    token_nodes: &'me mut TokensIterator<'content>,
    expected: &'static str,
    context: &ExpandContext,
    rule: ExpansionRule,
) -> Result<AtomicToken<'content>, ParseError> {
    token_nodes.with_expand_tracer(|_, tracer| tracer.start("atom"));

    let result = expand_atom_inner(token_nodes, expected, context, rule);

    token_nodes.with_expand_tracer(|_, tracer| match &result {
        Ok(result) => {
            tracer.add_result(result.clone());
            tracer.success();
        }

        Err(err) => tracer.failed(err),
    });

    result
}

/// If the caller of expand_atom throws away the returned atomic token returned, it
/// must use a checkpoint to roll it back.
fn expand_atom_inner<'me, 'content>(
    token_nodes: &'me mut TokensIterator<'content>,
    expected: &'static str,
    context: &ExpandContext,
    rule: ExpansionRule,
) -> Result<AtomicToken<'content>, ParseError> {
    if token_nodes.at_end() {
        match rule.allow_eof {
            true => {
                return Ok(UnspannedAtomicToken::Eof {
                    span: Span::unknown(),
                }
                .into_atomic_token(Span::unknown()))
            }
            false => return Err(ParseError::unexpected_eof("anything", Span::unknown())),
        }
    }

    // First, we'll need to handle the situation where more than one token corresponds
    // to a single atomic token

    // If treat_size_as_word, don't try to parse the head of the token stream
    // as a size.
    match rule.treat_size_as_word {
        true => {}
        false => match expand_syntax(&UnitShape, token_nodes, context) {
            // If the head of the stream isn't a valid unit, we'll try to parse
            // it again next as a word
            Err(_) => {}

            // But if it was a valid unit, we're done here
            Ok(UnitSyntax {
                unit: (number, unit),
                span,
            }) => return Ok(UnspannedAtomicToken::Size { number, unit }.into_atomic_token(span)),
        },
    }

    match rule.separate_members {
        false => {}
        true => {
            let mut next = token_nodes.peek_any();

            match next.node {
                Some(token) if token.is_word() => {
                    next.commit();
                    return Ok(UnspannedAtomicToken::Word { text: token.span() }
                        .into_atomic_token(token.span()));
                }

                Some(token) if token.is_int() => {
                    next.commit();
                    return Ok(UnspannedAtomicToken::Number {
                        number: RawNumber::Int(token.span()),
                    }
                    .into_atomic_token(token.span()));
                }

                _ => {}
            }
        }
    }

    // Try to parse the head of the stream as a bare path. A bare path includes
    // words as well as `.`s, connected together without whitespace.
    match expand_syntax(&BarePathShape, token_nodes, context) {
        // If we didn't find a bare path
        Err(_) => {}
        Ok(span) => {
            let next = token_nodes.peek_any();

            match next.node {
                Some(token) if token.is_pattern() => {
                    // if the very next token is a pattern, we're looking at a glob, not a
                    // word, and we should try to parse it as a glob next
                }

                _ => return Ok(UnspannedAtomicToken::Word { text: span }.into_atomic_token(span)),
            }
        }
    }

    // Try to parse the head of the stream as a pattern. A pattern includes
    // words, words with `*` as well as `.`s, connected together without whitespace.
    match expand_syntax(&BarePatternShape, token_nodes, context) {
        // If we didn't find a bare path
        Err(_) => {}
        Ok(span) => {
            return Ok(UnspannedAtomicToken::GlobPattern { pattern: span }.into_atomic_token(span))
        }
    }

    // The next token corresponds to at most one atomic token

    // We need to `peek` because `parse_single_node` doesn't cover all of the
    // cases that `expand_atom` covers. We should probably collapse the two
    // if possible.
    let peeked = token_nodes.peek_any().not_eof(expected)?;

    match peeked.node {
        TokenNode::Token(_) => {
            // handle this next
        }

        TokenNode::Error(error) => {
            peeked.commit();
            return Ok(UnspannedAtomicToken::Error {
                error: error.clone(),
            }
            .into_atomic_token(error.span));
        }

        // [ ... ]
        TokenNode::Delimited(Spanned {
            item:
                DelimitedNode {
                    delimiter: Delimiter::Square,
                    spans,
                    children,
                },
            span,
        }) => {
            peeked.commit();
            let span = *span;
            return Ok(UnspannedAtomicToken::SquareDelimited {
                nodes: children,
                spans: *spans,
            }
            .into_atomic_token(span));
        }

        TokenNode::Flag(Flag {
            kind: FlagKind::Shorthand,
            name,
            span,
        }) => {
            peeked.commit();
            return Ok(UnspannedAtomicToken::ShorthandFlag { name: *name }.into_atomic_token(*span));
        }

        TokenNode::Flag(Flag {
            kind: FlagKind::Longhand,
            name,
            span,
        }) => {
            peeked.commit();
            return Ok(UnspannedAtomicToken::ShorthandFlag { name: *name }.into_atomic_token(*span));
        }

        // If we see whitespace, process the whitespace according to the whitespace
        // handling rules
        TokenNode::Whitespace(span) => match rule.whitespace {
            // if whitespace is allowed, return a whitespace token
            WhitespaceHandling::AllowWhitespace => {
                peeked.commit();
                return Ok(
                    UnspannedAtomicToken::Whitespace { text: *span }.into_atomic_token(*span)
                );
            }

            // if whitespace is disallowed, return an error
            WhitespaceHandling::RejectWhitespace => {
                return Err(ParseError::mismatch(expected, "whitespace".spanned(*span)))
            }
        },

        other => {
            let span = peeked.node.span();

            peeked.commit();
            return Ok(UnspannedAtomicToken::Error {
                error: ShellError::type_error("token", other.type_name().spanned(span))
                    .spanned(span),
            }
            .into_atomic_token(span));
        }
    }

    parse_single_node(token_nodes, expected, |token, token_span, err| {
        Ok(match token {
            // First, the error cases. Each error case corresponds to a expansion rule
            // flag that can be used to allow the case

            // rule.allow_operator
            UnspannedToken::Operator(_) if !rule.allow_operator => return Err(err.error()),
            // rule.allow_external_command
            UnspannedToken::ExternalCommand(_) if !rule.allow_external_command => {
                return Err(ParseError::mismatch(
                    expected,
                    token.type_name().spanned(token_span),
                ))
            }
            // rule.allow_external_word
            UnspannedToken::ExternalWord if !rule.allow_external_word => {
                return Err(ParseError::mismatch(
                    expected,
                    "external word".spanned(token_span),
                ))
            }

            UnspannedToken::Number(number) => {
                UnspannedAtomicToken::Number { number }.into_atomic_token(token_span)
            }
            UnspannedToken::Operator(_) => {
                UnspannedAtomicToken::Operator { text: token_span }.into_atomic_token(token_span)
            }
            UnspannedToken::String(body) => {
                UnspannedAtomicToken::String { body }.into_atomic_token(token_span)
            }
            UnspannedToken::Variable(name) if name.slice(context.source) == "it" => {
                UnspannedAtomicToken::ItVariable { name }.into_atomic_token(token_span)
            }
            UnspannedToken::Variable(name) => {
                UnspannedAtomicToken::Variable { name }.into_atomic_token(token_span)
            }
            UnspannedToken::ExternalCommand(command) => {
                UnspannedAtomicToken::ExternalCommand { command }.into_atomic_token(token_span)
            }
            UnspannedToken::ExternalWord => UnspannedAtomicToken::ExternalWord { text: token_span }
                .into_atomic_token(token_span),
            UnspannedToken::GlobPattern => UnspannedAtomicToken::GlobPattern {
                pattern: token_span,
            }
            .into_atomic_token(token_span),
            UnspannedToken::Bare => {
                UnspannedAtomicToken::Word { text: token_span }.into_atomic_token(token_span)
            }
        })
    })
}
