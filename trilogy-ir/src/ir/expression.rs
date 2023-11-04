use super::*;
use crate::{Converter, Error};
use source_span::Span;
use trilogy_parser::{syntax, Spanned};

#[derive(Clone, Debug)]
pub struct Expression {
    pub span: Span,
    pub value: Value,
}

impl Expression {
    pub(super) fn convert(converter: &mut Converter, ast: syntax::Expression) -> Self {
        use syntax::Expression::*;
        match ast {
            Number(ast) => Self::number(ast.span(), crate::ir::Number::convert(*ast)),
            Character(ast) => Self::character(ast.span(), ast.value()),
            String(ast) => Self::string(ast.span(), ast.value()),
            Bits(ast) => Self::bits(ast.span(), crate::ir::Bits::convert(*ast)),
            Boolean(ast) => Self::boolean(ast.span(), ast.value()),
            Unit(ast) => Self::unit(ast.span()),
            Atom(ast) => Self::atom(ast.span(), ast.value()),
            Struct(ast) => Self::builtin(ast.span(), Builtin::Construct)
                .apply_to(ast.value.span(), Self::convert(converter, ast.value))
                .apply_to(
                    ast.atom.span(),
                    Self::atom(ast.atom.span(), ast.atom.value()),
                ),
            Array(ast) => {
                let start_span = ast.start_token().span;
                let span = ast.span();
                let elements = ast
                    .elements
                    .into_iter()
                    .map(|element| Element::convert_array(converter, element))
                    .collect::<Pack>();
                Self::builtin(start_span, Builtin::Array).apply_to(span, Self::pack(span, elements))
            }
            Set(ast) => {
                let start_span = ast.start_token().span;
                let span = ast.span();
                let elements = ast
                    .elements
                    .into_iter()
                    .map(|element| Element::convert_set(converter, element))
                    .collect::<Pack>();
                Self::builtin(start_span, Builtin::Set).apply_to(span, Self::pack(span, elements))
            }
            Record(ast) => {
                let start_span = ast.start_token().span;
                let span = ast.span();
                let elements = ast
                    .elements
                    .into_iter()
                    .map(|element| Element::convert_record(converter, element))
                    .collect::<Pack>();
                Self::builtin(start_span, Builtin::Record)
                    .apply_to(span, Self::pack(span, elements))
            }
            ArrayComprehension(ast) => {
                let start_span = ast.start_token().span;
                let span = ast.span();
                let iterator = Self::convert_iterator(converter, ast.query, ast.expression);
                Self::builtin(start_span, Builtin::Array).apply_to(span, iterator)
            }
            SetComprehension(ast) => {
                let start_span = ast.start_token().span;
                let span = ast.span();
                let iterator = Self::convert_iterator(converter, ast.query, ast.expression);
                Self::builtin(start_span, Builtin::Set).apply_to(span, iterator)
            }
            RecordComprehension(ast) => {
                let span = ast.span();
                let start_span = ast.start_token().span;
                let iter_span = ast
                    .query
                    .span()
                    .union(ast.key_expression.span())
                    .union(ast.expression.span());
                converter.push_scope();
                let query = Query::convert(converter, ast.query);
                let key = Self::convert(converter, ast.key_expression);
                let value = Self::convert(converter, ast.expression);
                converter.pop_scope();
                let iterator = Self::iterator(
                    iter_span,
                    query,
                    Self::mapping(key.span.union(value.span), key, value),
                );
                Self::builtin(start_span, Builtin::Set).apply_to(span, iterator)
            }
            IteratorComprehension(ast) => {
                Self::convert_iterator(converter, ast.query, ast.expression)
            }
            Reference(ast) => Self::reference(
                ast.span(),
                Identifier::declared(converter, &ast).unwrap_or_else(|| {
                    converter.error(Error::UnboundIdentifier {
                        name: (*ast).clone(),
                    });
                    Identifier::unresolved(converter, *ast)
                }),
            ),
            Keyword(ast) => Builtin::convert(*ast),
            Application(ast) => Self::application(
                ast.span(),
                Self::convert(converter, ast.function),
                Self::convert(converter, ast.argument),
            ),
            Call(ast) => {
                let span = ast.span();
                let argument_span = ast.start_token().span.union(ast.end_token().span);
                let proc = Self::convert(converter, ast.procedure);
                let arguments = ast
                    .arguments
                    .into_iter()
                    .map(|ast| Self::convert(converter, ast))
                    .collect::<Pack>();
                let arguments = Self::pack(argument_span, arguments);
                Self::application(span, proc, arguments)
            }
            Binary(ast) => {
                let span = ast.span();
                let lhs_span = ast.operator.span().union(ast.lhs.span());
                let op = Builtin::convert_binary(ast.operator);
                op.apply_to(lhs_span, Self::convert(converter, ast.lhs))
                    .apply_to(span, Self::convert(converter, ast.rhs))
            }
            Unary(ast) => {
                let span = ast.span();
                let op = Builtin::convert_unary(ast.operator);
                op.apply_to(span, Self::convert(converter, ast.operand))
            }
            Let(ast) => crate::ir::Let::convert(converter, *ast),
            IfElse(ast) => crate::ir::IfElse::convert_expression(converter, *ast),
            Match(ast) => crate::ir::Match::convert_expression(converter, *ast),
            Is(ast) => Self::application(
                ast.span(),
                Self::builtin(ast.is_token().span, Builtin::Is),
                Self::convert_query(converter, ast.query),
            ),
            End(ast) => Self::end(ast.span()),
            Exit(ast) => Self::application(
                ast.span(),
                Self::builtin(ast.exit_token().span, Builtin::Exit),
                Self::convert(converter, ast.expression),
            ),
            Resume(ast) => Self::application(
                ast.span(),
                Self::builtin(ast.resume_token().span, Builtin::Resume),
                Self::convert(converter, ast.expression),
            ),
            Cancel(ast) => Self::application(
                ast.span(),
                Self::builtin(ast.cancel_token().span, Builtin::Cancel),
                Self::convert(converter, ast.expression),
            ),
            Return(ast) => Self::application(
                ast.span(),
                Self::builtin(ast.return_token().span, Builtin::Return),
                Self::convert(converter, ast.expression),
            ),
            Break(ast) => Self::application(
                ast.span(),
                Self::builtin(ast.break_token().span, Builtin::Break),
                Self::convert(converter, ast.expression),
            ),
            Continue(ast) => Self::application(
                ast.span(),
                Self::builtin(ast.continue_token().span, Builtin::Continue),
                Self::convert(converter, ast.expression),
            ),
            Fn(ast) => Self::function(ast.span(), Function::convert_fn(converter, *ast)),
            Do(ast) => Self::procedure(ast.span(), Procedure::convert_do(converter, *ast)),
            Template(ast) => Self::convert_template(converter, *ast),
            Handled(ast) => crate::ir::Handled::convert_expression(converter, *ast),
            Parenthesized(ast) => Self::convert(converter, ast.expression),
            ModuleAccess(ast) => {
                let span = ast.span();
                let lhs_span = ast.lhs.span().union(ast.access_token().span());
                Self::builtin(ast.access_token().span, Builtin::ModuleAccess)
                    .apply_to(lhs_span, Self::convert(converter, ast.lhs))
                    .apply_to(span, Self::dynamic(ast.rhs))
            }
        }
    }

    pub(super) fn convert_block(converter: &mut Converter, ast: syntax::Block) -> Self {
        let span = ast.span();
        converter.push_scope();
        let sequence = Self::convert_sequence(converter, &mut ast.statements.into_iter());
        converter.pop_scope();
        Self::sequence(span, sequence)
    }

    pub(super) fn convert_sequence(
        converter: &mut Converter,
        statements: &mut impl std::iter::Iterator<Item = syntax::Statement>,
    ) -> Vec<Self> {
        let mut sequence = vec![];
        Self::convert_sequence_into(converter, statements, &mut sequence);
        sequence
    }

    fn convert_sequence_into(
        converter: &mut Converter,
        statements: &mut impl std::iter::Iterator<Item = syntax::Statement>,
        sequence: &mut Vec<Self>,
    ) {
        let statement = match statements.next() {
            Some(ast) => Self::convert_statement(converter, ast, statements),
            None => return,
        };
        sequence.push(statement);
        Self::convert_sequence_into(converter, statements, sequence);
    }

    fn convert_statement(
        converter: &mut Converter,
        ast: syntax::Statement,
        rest: &mut impl std::iter::Iterator<Item = syntax::Statement>,
    ) -> Self {
        use syntax::Statement::*;
        match ast {
            Let(ast) => crate::ir::Let::convert_statement(converter, *ast, rest),
            Assignment(ast) => crate::ir::Assignment::convert(converter, *ast),
            FunctionAssignment(ast) => crate::ir::Assignment::convert_function(converter, *ast),
            If(ast) => IfElse::convert_statement(converter, *ast),
            Match(ast) => crate::ir::Match::convert_statement(converter, *ast),
            While(ast) => crate::ir::While::convert(converter, *ast),
            For(ast) => Self::convert_for_statement(converter, *ast),
            Break(ast) => Self::application(
                ast.span(),
                Self::builtin(ast.span(), Builtin::Break),
                Self::unit(ast.span()),
            ),
            Continue(ast) => Self::application(
                ast.span(),
                Self::builtin(ast.span(), Builtin::Continue),
                Self::unit(ast.span()),
            ),
            Resume(ast) => {
                let span = ast.span();
                Self::application(
                    span,
                    Self::builtin(ast.resume_token().span, Builtin::Resume),
                    ast.expression
                        .map(|ast| Self::convert(converter, ast))
                        .unwrap_or_else(|| Self::unit(span)),
                )
            }
            Cancel(ast) => {
                let span = ast.span();
                Self::application(
                    span,
                    Self::builtin(ast.cancel_token().span, Builtin::Cancel),
                    ast.expression
                        .map(|ast| Self::convert(converter, ast))
                        .unwrap_or_else(|| Self::unit(span)),
                )
            }
            Return(ast) => {
                let span = ast.span();
                Self::application(
                    span,
                    Self::builtin(ast.return_token().span, Builtin::Return),
                    ast.expression
                        .map(|ast| Self::convert(converter, ast))
                        .unwrap_or_else(|| Self::unit(span)),
                )
            }
            End(ast) => Self::end(ast.span()),
            Exit(ast) => Self::application(
                ast.span(),
                Self::builtin(ast.exit_token().span, Builtin::Exit),
                Self::convert(converter, ast.expression),
            ),
            Yield(ast) => Self::application(
                ast.span(),
                Self::builtin(ast.yield_token().span, Builtin::Yield),
                Self::convert(converter, ast.expression),
            ),
            Expression(ast) => Self::convert(converter, *ast),
            Assert(ast) => Self::assert(ast.span(), crate::ir::Assert::convert(converter, *ast)),
            Handled(ast) => crate::ir::Handled::convert_block(converter, *ast),
            Block(ast) => Self::convert_block(converter, *ast),
        }
    }

    pub(super) fn convert_query(converter: &mut Converter, ast: syntax::Query) -> Self {
        let span = ast.span();
        let query = Query::convert(converter, ast);
        Self::query(span, query)
    }

    pub(super) fn convert_pattern(converter: &mut Converter, ast: syntax::Pattern) -> Self {
        use syntax::Pattern::*;
        match ast {
            Conjunction(ast) => Self::conjunction(
                ast.span(),
                Self::convert_pattern(converter, ast.lhs),
                Self::convert_pattern(converter, ast.rhs),
            ),
            Disjunction(ast) => Self::disjunction(
                ast.span(),
                Self::convert_pattern(converter, ast.lhs),
                Self::convert_pattern(converter, ast.rhs),
            ),
            Number(ast) => Self::number(ast.span(), crate::ir::Number::convert(*ast)),
            Character(ast) => Self::character(ast.span(), ast.value()),
            String(ast) => Self::string(ast.span(), ast.value()),
            Bits(ast) => Self::bits(ast.span(), crate::ir::Bits::convert(*ast)),
            Boolean(ast) => Self::boolean(ast.span(), ast.value()),
            Unit(ast) => Self::unit(ast.span()),
            Atom(ast) => Self::atom(ast.span(), ast.value()),
            Wildcard(ast) => Self::wildcard(ast.span()),
            Negative(ast) => Self::builtin(ast.minus_token().span, Builtin::Negate)
                .apply_to(ast.span(), Self::convert_pattern(converter, ast.pattern)),
            Glue(ast) => {
                let glue_span = ast.glue_token().span;
                let lhs_span = ast.lhs.span();
                let span = ast.span();
                Self::builtin(glue_span, Builtin::Glue)
                    .apply_to(
                        lhs_span.union(glue_span),
                        Self::convert_pattern(converter, ast.lhs),
                    )
                    .apply_to(span, Self::convert_pattern(converter, ast.rhs))
            }
            Struct(ast) => Self::builtin(ast.span(), Builtin::Construct)
                .apply_to(
                    ast.pattern.span(),
                    Self::convert_pattern(converter, ast.pattern),
                )
                .apply_to(
                    ast.atom.span(),
                    Self::atom(ast.atom.span(), ast.atom.value()),
                ),
            Tuple(ast) => {
                let cons_span = ast.cons_token().span;
                let lhs_span = ast.lhs.span();
                let span = ast.span();
                Self::builtin(cons_span, Builtin::Cons)
                    .apply_to(
                        lhs_span.union(cons_span),
                        Self::convert_pattern(converter, ast.lhs),
                    )
                    .apply_to(span, Self::convert_pattern(converter, ast.rhs))
            }
            Array(ast) => {
                let start_span = ast.start_token().span;
                let span = ast.span();
                let mut elements: Pack = ast
                    .head
                    .into_iter()
                    .map(|element| Self::convert_pattern(converter, element))
                    .map(Element::from)
                    .collect();
                elements.extend(
                    ast.rest
                        .into_iter()
                        .map(|element| Self::convert_pattern(converter, element))
                        .map(Element::spread),
                );
                elements.extend(
                    ast.tail
                        .into_iter()
                        .map(|element| Self::convert_pattern(converter, element))
                        .map(Element::from),
                );
                Self::builtin(start_span, Builtin::Array).apply_to(span, Self::pack(span, elements))
            }
            Set(ast) => {
                let start_span = ast.start_token().span;
                let span = ast.span();
                let mut elements: Pack = ast
                    .elements
                    .into_iter()
                    .map(|element| Self::convert_pattern(converter, element))
                    .map(Element::from)
                    .collect();
                elements.extend(
                    ast.rest
                        .into_iter()
                        .map(|element| Self::convert_pattern(converter, element))
                        .map(Element::spread),
                );
                Self::builtin(start_span, Builtin::Set).apply_to(span, Self::pack(span, elements))
            }
            Record(ast) => {
                let start_span = ast.start_token().span;
                let span = ast.span();
                let mut elements: Pack = ast
                    .elements
                    .into_iter()
                    .map(|(key, value)| {
                        Self::mapping(
                            key.span().union(value.span()),
                            Self::convert_pattern(converter, key),
                            Self::convert_pattern(converter, value),
                        )
                    })
                    .map(Element::from)
                    .collect();
                elements.extend(
                    ast.rest
                        .into_iter()
                        .map(|element| Self::convert_pattern(converter, element))
                        .map(Element::spread),
                );
                Self::builtin(start_span, Builtin::Record)
                    .apply_to(span, Self::pack(span, elements))
            }
            Pinned(ast) => Identifier::declared(converter, &ast.identifier)
                .map(|identifier| {
                    Self::builtin(ast.span(), Builtin::Pin)
                        .apply_to(ast.span(), Self::reference(ast.span(), identifier))
                })
                .unwrap_or_else(|| {
                    converter.error(Error::UnboundIdentifier {
                        name: ast.identifier.clone(),
                    });
                    Self::reference(
                        ast.identifier.span(),
                        Identifier::unresolved(converter, ast.identifier),
                    )
                }),
            Binding(ast) => {
                Self::reference(ast.span(), Identifier::declare_binding(converter, *ast))
            }
            Parenthesized(ast) => Self::convert_pattern(converter, ast.pattern),
        }
    }

    fn convert_for_statement(converter: &mut Converter, ast: syntax::ForStatement) -> Self {
        let else_block = ast
            .else_block
            .map(|ast| Expression::convert_block(converter, ast));

        else_block
            .into_iter()
            .chain(ast.branches.into_iter().rev().map(|branch| {
                let for_span = branch.for_token().span;
                let span = branch.span();
                converter.push_scope();
                let query = Query::convert(converter, branch.query);
                let value = Expression::convert_block(converter, branch.body);
                converter.pop_scope();
                Expression::builtin(for_span, Builtin::For)
                    .apply_to(span, Expression::iterator(span, query, value))
            }))
            .reduce(|if_none, case| {
                let case_span = case.span;
                Expression::if_else(
                    case.span.union(if_none.span),
                    IfElse::new(case, Expression::boolean(case_span, true), if_none),
                )
            })
            .unwrap()
    }

    fn convert_iterator(
        converter: &mut Converter,
        query: syntax::Query,
        expression: syntax::Expression,
    ) -> Self {
        let span = query.span().union(expression.span());
        converter.push_scope();
        let query = Query::convert(converter, query);
        let body = Self::convert(converter, expression);
        converter.pop_scope();
        Self::iterator(span, query, body)
    }

    fn convert_template(converter: &mut Converter, ast: syntax::Template) -> Self {
        let span = ast.span();
        let prefix = Self::string(ast.prefix_token().span, ast.prefix());
        match ast.tag {
            Some(tag) => {
                let (strings, interpolations) = ast
                    .segments
                    .into_iter()
                    .map(|seg| {
                        let suffix = Self::string(seg.suffix_token().span, seg.suffix());
                        let interpolation = Self::convert(converter, seg.interpolation);
                        (interpolation, suffix)
                    })
                    .fold(
                        (vec![prefix], vec![]),
                        |(mut strings, mut interpolations), (interpolation, suffix)| {
                            strings.push(suffix);
                            interpolations.push(interpolation);
                            (strings, interpolations)
                        },
                    );

                let tag = Identifier::declared(converter, &tag)
                    .map(|tag| Expression::reference(tag.span, tag))
                    .unwrap_or_else(|| {
                        converter.error(Error::UnboundIdentifier { name: tag.clone() });
                        Expression::reference(tag.span(), Identifier::unresolved(converter, tag))
                    });
                let strings = Self::builtin(span, Builtin::Array)
                    .apply_to(span, Self::pack(span, Pack::from_iter(strings)));
                let interpolations = Self::builtin(span, Builtin::Array)
                    .apply_to(span, Self::pack(span, Pack::from_iter(interpolations)));
                tag.apply_to(span, strings).apply_to(span, interpolations)
            }
            None => {
                let span = ast.span();
                let prefix = Self::string(ast.prefix_token().span, ast.prefix());
                ast.segments
                    .into_iter()
                    .map(|seg| {
                        let suffix = Self::string(seg.suffix_token().span, seg.suffix());
                        let interpolation = Self::convert(converter, seg.interpolation);
                        (interpolation, suffix)
                    })
                    .fold(prefix, |expr, (interpolation, suffix)| {
                        Self::builtin(span, Builtin::Glue)
                            .apply_to(
                                span,
                                Self::builtin(span, Builtin::Glue)
                                    .apply_to(span, expr)
                                    .apply_to(span, interpolation),
                            )
                            .apply_to(span, suffix)
                    })
            }
        }
    }

    pub(super) fn new(span: Span, value: Value) -> Self {
        Self { span, value }
    }

    pub(super) fn boolean(span: Span, value: bool) -> Self {
        Self::new(span, Value::Boolean(value))
    }

    pub(super) fn number(span: Span, value: Number) -> Self {
        Self::new(span, Value::Number(Box::new(value)))
    }

    pub(super) fn string(span: Span, value: String) -> Self {
        Self::new(span, Value::String(value))
    }

    pub(super) fn character(span: Span, value: char) -> Self {
        Self::new(span, Value::Character(value))
    }

    pub(super) fn bits(span: Span, value: Bits) -> Self {
        Self::new(span, Value::Bits(value))
    }

    pub(super) fn atom(span: Span, value: String) -> Self {
        Self::new(span, Value::Atom(value))
    }

    pub(super) fn unit(span: Span) -> Self {
        Self::new(span, Value::Unit)
    }

    pub(super) fn wildcard(span: Span) -> Self {
        Self::new(span, Value::Wildcard)
    }

    pub(super) fn pack(span: Span, pack: Pack) -> Self {
        Self::new(span, Value::Pack(Box::new(pack)))
    }

    pub(super) fn mapping(span: Span, key: Expression, value: Expression) -> Self {
        Self::new(span, Value::Mapping(Box::new((key, value))))
    }

    pub(super) fn r#let(span: Span, body: Let) -> Self {
        Self::new(span, Value::Let(Box::new(body)))
    }

    pub(super) fn handled(span: Span, handled: Handled) -> Self {
        Self::new(span, Value::Handled(Box::new(handled)))
    }

    pub(super) fn iterator(span: Span, query: Query, value: Expression) -> Self {
        Self::new(span, Value::Iterator(Box::new(Iterator::new(query, value))))
    }

    pub(super) fn assignment(span: Span, assignment: Assignment) -> Self {
        Self::new(span, Value::Assignment(Box::new(assignment)))
    }

    pub(super) fn if_else(span: Span, if_else: IfElse) -> Self {
        Self::new(span, Value::IfElse(Box::new(if_else)))
    }

    pub(super) fn r#while(span: Span, body: While) -> Self {
        Self::new(span, Value::While(Box::new(body)))
    }

    pub(super) fn r#match(span: Span, body: Match) -> Self {
        Self::new(span, Value::Match(Box::new(body)))
    }

    pub(super) fn end(span: Span) -> Self {
        Self::new(span, Value::End)
    }

    pub(super) fn dynamic(identifier: syntax::Identifier) -> Self {
        Self::new(identifier.span(), Value::Dynamic(Box::new(identifier)))
    }

    pub(super) fn sequence(span: Span, sequence: Vec<Expression>) -> Self {
        Self::new(span, Value::Sequence(sequence))
    }

    pub(super) fn query(span: Span, query: Query) -> Self {
        Self::new(span, Value::Query(Box::new(query)))
    }

    pub(super) fn assert(span: Span, assert: Assert) -> Self {
        Self::new(span, Value::Assert(Box::new(assert)))
    }

    pub(super) fn application(span: Span, lhs: Expression, rhs: Expression) -> Self {
        Self::new(
            span,
            Value::Application(Box::new(Application::new(lhs, rhs))),
        )
    }

    pub(super) fn builtin(span: Span, builtin: Builtin) -> Self {
        Self::new(span, Value::Builtin(builtin))
    }

    pub(super) fn reference(span: Span, identifier: Identifier) -> Self {
        Self::new(span, Value::Reference(Box::new(identifier)))
    }

    pub(super) fn function(span: Span, function: Function) -> Self {
        Self::new(span, Value::Fn(Box::new(function)))
    }

    pub(super) fn procedure(span: Span, procedure: Procedure) -> Self {
        Self::new(span, Value::Do(Box::new(procedure)))
    }

    pub(super) fn apply_to(self, span: Span, rhs: Expression) -> Self {
        Self::application(span, self, rhs)
    }

    pub(super) fn in_let(self, span: Span, query: Query) -> Self {
        Expression::r#let(span, Let::new(query, self))
    }

    pub(super) fn and(self, span: Span, other: Expression) -> Self {
        Expression::conjunction(span, other, self)
    }

    pub(super) fn conjunction(span: Span, lhs: Expression, rhs: Expression) -> Self {
        Expression::new(span, Value::Conjunction(Box::new((lhs, rhs))))
    }

    pub(super) fn disjunction(span: Span, lhs: Expression, rhs: Expression) -> Self {
        Expression::new(span, Value::Disjunction(Box::new((lhs, rhs))))
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    Builtin(Builtin),
    Pack(Box<Pack>),
    Sequence(Vec<Expression>),
    Assignment(Box<Assignment>),
    Mapping(Box<(Expression, Expression)>),
    Number(Box<Number>),
    Character(char),
    String(String),
    Bits(Bits),
    Boolean(bool),
    Unit,
    Conjunction(Box<(Expression, Expression)>),
    Disjunction(Box<(Expression, Expression)>),
    Wildcard,
    Atom(String),
    Query(Box<Query>),
    Iterator(Box<Iterator>),
    While(Box<While>),
    Application(Box<Application>),
    Let(Box<Let>),
    IfElse(Box<IfElse>),
    Match(Box<Match>),
    Fn(Box<Function>),
    Do(Box<Procedure>),
    Handled(Box<Handled>),
    Reference(Box<Identifier>),
    Dynamic(Box<syntax::Identifier>),
    Assert(Box<Assert>),
    End,
}
