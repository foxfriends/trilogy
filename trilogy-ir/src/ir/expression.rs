use super::*;
use crate::{Analyzer, Error, Id};
use source_span::Span;
use trilogy_parser::{syntax, Spanned};

#[derive(Clone, Debug)]
pub struct Expression {
    pub span: Span,
    pub value: Value,
}

impl Expression {
    pub fn bindings(&self) -> impl std::iter::Iterator<Item = Id> + '_ {
        self.value.bindings()
    }

    pub(super) fn convert(analyzer: &mut Analyzer, ast: syntax::Expression) -> Self {
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
                .apply_to(
                    ast.atom.span(),
                    Self::atom(ast.atom.span(), ast.atom.value()),
                )
                .apply_to(ast.value.span(), Self::convert(analyzer, ast.value)),
            Array(ast) => {
                let start_span = ast.start_token().span;
                let span = ast.span();
                let elements = ast
                    .elements
                    .into_iter()
                    .map(|element| Element::convert_array(analyzer, element))
                    .collect::<Pack>();
                Self::builtin(start_span, Builtin::Array).apply_to(span, Self::pack(span, elements))
            }
            Set(ast) => {
                let start_span = ast.start_token().span;
                let span = ast.span();
                let elements = ast
                    .elements
                    .into_iter()
                    .map(|element| Element::convert_set(analyzer, element))
                    .collect::<Pack>();
                Self::builtin(start_span, Builtin::Set).apply_to(span, Self::pack(span, elements))
            }
            Record(ast) => {
                let start_span = ast.start_token().span;
                let span = ast.span();
                let elements = ast
                    .elements
                    .into_iter()
                    .map(|element| Element::convert_record(analyzer, element))
                    .collect::<Pack>();
                Self::builtin(start_span, Builtin::Record)
                    .apply_to(span, Self::pack(span, elements))
            }
            ArrayComprehension(ast) => {
                let start_span = ast.start_token().span;
                let span = ast.span();
                let iterator = Self::convert_iterator(analyzer, ast.query, ast.expression);
                Self::builtin(start_span, Builtin::Array).apply_to(span, iterator)
            }
            SetComprehension(ast) => {
                let start_span = ast.start_token().span;
                let span = ast.span();
                let iterator = Self::convert_iterator(analyzer, ast.query, ast.expression);
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
                analyzer.push_scope();
                let query = Self::convert_query(analyzer, ast.query);
                let key = Self::convert(analyzer, ast.key_expression);
                let value = Self::convert(analyzer, ast.expression);
                analyzer.pop_scope();
                let iterator = Self::iterator(
                    iter_span,
                    query,
                    Self::mapping(key.span.union(value.span), key, value),
                );
                Self::builtin(start_span, Builtin::Set).apply_to(span, iterator)
            }
            IteratorComprehension(ast) => {
                Self::convert_iterator(analyzer, ast.query, ast.expression)
            }
            Reference(ast) => Self::convert_path(analyzer, *ast),
            Keyword(ast) => Builtin::convert(*ast),
            Application(ast) => Self::application(
                ast.span(),
                Self::convert(analyzer, ast.function),
                Self::convert(analyzer, ast.argument),
            ),
            Call(ast) => {
                let span = ast.span();
                let argument_span = ast.start_token().span.union(ast.end_token().span);
                let proc = Self::convert(analyzer, ast.procedure);
                let arguments = ast
                    .arguments
                    .into_iter()
                    .map(|ast| Self::convert(analyzer, ast))
                    .collect::<Pack>();
                let arguments = Self::pack(argument_span, arguments);
                Self::application(span, proc, arguments)
            }
            Binary(ast) => {
                let span = ast.span();
                let lhs_span = ast.operator.span().union(ast.lhs.span());
                let op = Builtin::convert_binary(ast.operator);
                op.apply_to(lhs_span, Self::convert(analyzer, ast.lhs))
                    .apply_to(span, Self::convert(analyzer, ast.rhs))
            }
            Unary(ast) => {
                let span = ast.span();
                let op = Builtin::convert_unary(ast.operator);
                op.apply_to(span, Self::convert(analyzer, ast.operand))
            }
            Let(ast) => crate::ir::Let::convert(analyzer, *ast),
            IfElse(ast) => crate::ir::IfElse::convert_expression(analyzer, *ast),
            Match(ast) => crate::ir::Match::convert_expression(analyzer, *ast),
            Is(ast) => Self::application(
                ast.span(),
                Self::builtin(ast.is_token().span, Builtin::Is),
                Self::convert_query(analyzer, ast.query),
            ),
            End(ast) => Self::end(ast.span()),
            Exit(ast) => Self::application(
                ast.span(),
                Self::builtin(ast.exit_token().span, Builtin::Exit),
                Self::convert(analyzer, ast.expression),
            ),
            Resume(ast) => Self::application(
                ast.span(),
                Self::builtin(ast.resume_token().span, Builtin::Resume),
                Self::convert(analyzer, ast.expression),
            ),
            Cancel(ast) => Self::application(
                ast.span(),
                Self::builtin(ast.cancel_token().span, Builtin::Cancel),
                Self::convert(analyzer, ast.expression),
            ),
            Return(ast) => Self::application(
                ast.span(),
                Self::builtin(ast.return_token().span, Builtin::Return),
                Self::convert(analyzer, ast.expression),
            ),
            Break(ast) => Self::application(
                ast.span(),
                Self::builtin(ast.break_token().span, Builtin::Break),
                Self::convert(analyzer, ast.expression),
            ),
            Continue(ast) => Self::application(
                ast.span(),
                Self::builtin(ast.continue_token().span, Builtin::Continue),
                Self::convert(analyzer, ast.expression),
            ),
            Fn(ast) => Self::function(ast.span(), Function::convert_fn(analyzer, *ast)),
            Do(ast) => Self::procedure(ast.span(), Procedure::convert_do(analyzer, *ast)),
            Template(ast) => Self::convert_template(analyzer, *ast),
            Handled(ast) => crate::ir::Handled::convert_expression(analyzer, *ast),
            Parenthesized(ast) => Self::convert(analyzer, ast.expression),
            Module(ast) => Self::convert_module_path(analyzer, *ast),
        }
    }

    pub(super) fn convert_block(analyzer: &mut Analyzer, ast: syntax::Block) -> Self {
        let span = ast.span();
        analyzer.push_scope();
        let sequence = Self::convert_sequence(analyzer, &mut ast.statements.into_iter());
        analyzer.pop_scope();
        Self::sequence(span, sequence)
    }

    pub(super) fn convert_sequence(
        analyzer: &mut Analyzer,
        statements: &mut impl std::iter::Iterator<Item = syntax::Statement>,
    ) -> Vec<Self> {
        let mut sequence = vec![];
        Self::convert_sequence_into(analyzer, statements, &mut sequence);
        sequence
    }

    fn convert_sequence_into(
        analyzer: &mut Analyzer,
        statements: &mut impl std::iter::Iterator<Item = syntax::Statement>,
        sequence: &mut Vec<Self>,
    ) {
        let statement = match statements.next() {
            Some(ast) => Self::convert_statement(analyzer, ast, statements),
            None => return,
        };
        sequence.push(statement);
        Self::convert_sequence_into(analyzer, statements, sequence);
    }

    fn convert_statement(
        analyzer: &mut Analyzer,
        ast: syntax::Statement,
        rest: &mut impl std::iter::Iterator<Item = syntax::Statement>,
    ) -> Self {
        use syntax::Statement::*;
        match ast {
            Let(ast) => crate::ir::Let::convert_statement(analyzer, *ast, rest),
            Assignment(ast) => crate::ir::Assignment::convert(analyzer, *ast),
            FunctionAssignment(ast) => crate::ir::Assignment::convert_function(analyzer, *ast),
            If(ast) => IfElse::convert_statement(analyzer, *ast),
            Match(ast) => crate::ir::Match::convert_statement(analyzer, *ast),
            While(ast) => crate::ir::While::convert(analyzer, *ast),
            For(ast) => Self::convert_for_statement(analyzer, *ast),
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
                        .map(|ast| Self::convert(analyzer, ast))
                        .unwrap_or_else(|| Self::unit(span)),
                )
            }
            Cancel(ast) => {
                let span = ast.span();
                Self::application(
                    span,
                    Self::builtin(ast.cancel_token().span, Builtin::Cancel),
                    ast.expression
                        .map(|ast| Self::convert(analyzer, ast))
                        .unwrap_or_else(|| Self::unit(span)),
                )
            }
            Return(ast) => {
                let span = ast.span();
                Self::application(
                    span,
                    Self::builtin(ast.return_token().span, Builtin::Return),
                    ast.expression
                        .map(|ast| Self::convert(analyzer, ast))
                        .unwrap_or_else(|| Self::unit(span)),
                )
            }
            End(ast) => Self::end(ast.span()),
            Exit(ast) => Self::application(
                ast.span(),
                Self::builtin(ast.exit_token().span, Builtin::Exit),
                Self::convert(analyzer, ast.expression),
            ),
            Yield(ast) => Self::application(
                ast.span(),
                Self::builtin(ast.yield_token().span, Builtin::Yield),
                Self::convert(analyzer, ast.expression),
            ),
            Expression(ast) => Self::convert(analyzer, *ast),
            Assert(ast) => Self::assert(ast.span(), crate::ir::Assert::convert(analyzer, *ast)),
            Handled(ast) => crate::ir::Handled::convert_block(analyzer, *ast),
            Block(ast) => Self::convert_block(analyzer, *ast),
        }
    }

    pub(super) fn convert_query(analyzer: &mut Analyzer, ast: syntax::Query) -> Self {
        let span = ast.span();
        let query = Query::convert(analyzer, ast);
        Self::query(span, query)
    }

    pub(super) fn convert_pattern(analyzer: &mut Analyzer, ast: syntax::Pattern) -> Self {
        use syntax::Pattern::*;
        match ast {
            Conjunction(ast) => Self::conjunction(
                ast.span(),
                Self::convert_pattern(analyzer, ast.lhs),
                Self::convert_pattern(analyzer, ast.rhs),
            ),
            Disjunction(ast) => Self::disjunction(
                ast.span(),
                Self::convert_pattern(analyzer, ast.lhs),
                Self::convert_pattern(analyzer, ast.rhs),
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
                .apply_to(ast.span(), Self::convert_pattern(analyzer, ast.pattern)),
            Glue(ast) => {
                let glue_span = ast.glue_token().span;
                let lhs_span = ast.lhs.span();
                let span = ast.span();
                Self::builtin(glue_span, Builtin::Glue)
                    .apply_to(
                        lhs_span.union(glue_span),
                        Self::convert_pattern(analyzer, ast.lhs),
                    )
                    .apply_to(span, Self::convert_pattern(analyzer, ast.rhs))
            }
            Struct(ast) => Self::builtin(ast.span(), Builtin::Construct)
                .apply_to(
                    ast.atom.span(),
                    Self::atom(ast.atom.span(), ast.atom.value()),
                )
                .apply_to(
                    ast.pattern.span(),
                    Self::convert_pattern(analyzer, ast.pattern),
                ),
            Tuple(ast) => {
                let cons_span = ast.cons_token().span;
                let lhs_span = ast.lhs.span();
                let span = ast.span();
                Self::builtin(cons_span, Builtin::Cons)
                    .apply_to(
                        lhs_span.union(cons_span),
                        Self::convert_pattern(analyzer, ast.lhs),
                    )
                    .apply_to(span, Self::convert_pattern(analyzer, ast.rhs))
            }
            Array(ast) => {
                let start_span = ast.start_token().span;
                let span = ast.span();
                let mut elements: Pack = ast
                    .head
                    .into_iter()
                    .map(|element| Self::convert_pattern(analyzer, element))
                    .map(Element::from)
                    .collect();
                elements.extend(
                    ast.rest
                        .into_iter()
                        .map(|element| Self::convert_pattern(analyzer, element))
                        .map(Element::spread),
                );
                elements.extend(
                    ast.tail
                        .into_iter()
                        .map(|element| Self::convert_pattern(analyzer, element))
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
                    .map(|element| Self::convert_pattern(analyzer, element))
                    .map(Element::from)
                    .collect();
                elements.extend(
                    ast.rest
                        .into_iter()
                        .map(|element| Self::convert_pattern(analyzer, element))
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
                            Self::convert_pattern(analyzer, key),
                            Self::convert_pattern(analyzer, value),
                        )
                    })
                    .map(Element::from)
                    .collect();
                elements.extend(
                    ast.rest
                        .into_iter()
                        .map(|element| Self::convert_pattern(analyzer, element))
                        .map(Element::spread),
                );
                Self::builtin(start_span, Builtin::Record)
                    .apply_to(span, Self::pack(span, elements))
            }
            Pinned(ast) => Identifier::declared(analyzer, &ast.identifier)
                .map(|identifier| {
                    Self::builtin(ast.span(), Builtin::Pin)
                        .apply_to(ast.span(), Self::reference(ast.span(), identifier))
                })
                .unwrap_or_else(|| {
                    analyzer.error(Error::UnboundIdentifier {
                        name: ast.identifier.clone(),
                    });
                    // TODO: is dynamic the best way?
                    Self::dynamic(ast.identifier)
                }),
            Binding(ast) => {
                Self::reference(ast.span(), Identifier::declare(analyzer, ast.identifier))
            }
            Parenthesized(ast) => Self::convert_pattern(analyzer, ast.pattern),
        }
    }

    pub(super) fn convert_module_path(analyzer: &mut Analyzer, ast: syntax::ModulePath) -> Self {
        let value = Self::convert_module_reference(analyzer, ast.first);
        ast.modules.into_iter().fold(value, |module, (token, ast)| {
            let module_span = module.span;
            let module = Self::builtin(token.span, Builtin::ModuleAccess)
                .apply_to(module_span.union(token.span), module)
                .apply_to(
                    module_span.union(ast.name.span()),
                    Expression::dynamic(ast.name),
                );
            ast.arguments.into_iter().fold(module, |function, ast| {
                let span = function.span.union(ast.span());
                function.apply_to(span, Expression::convert(analyzer, ast))
            })
        })
    }

    pub(super) fn convert_path(analyzer: &mut Analyzer, ast: syntax::Path) -> Self {
        let span = ast.span();
        let join_token = ast.join_token().map(|token| token.span);
        match ast.module {
            Some(module) => {
                let module_span = module.span();
                let join_span = join_token.unwrap();
                Self::builtin(join_span, Builtin::ModuleAccess)
                    .apply_to(
                        module_span.union(join_span),
                        Self::convert_module_path(analyzer, module),
                    )
                    .apply_to(span, Self::dynamic(ast.member))
            }
            None => Identifier::declared(analyzer, &ast.member)
                .map(|identifier| Self::reference(span, identifier))
                .unwrap_or_else(|| {
                    analyzer.error(Error::UnboundIdentifier {
                        name: ast.member.clone(),
                    });
                    // TODO: is dynamic the best way?
                    Self::dynamic(ast.member)
                }),
        }
    }

    fn convert_module_reference(analyzer: &mut Analyzer, ast: syntax::ModuleReference) -> Self {
        let id = Identifier::declared(analyzer, &ast.name).unwrap_or_else(|| {
            analyzer.error(Error::UnknownModule {
                name: ast.name.clone(),
            });
            Identifier::declare(analyzer, ast.name.clone())
        });
        ast.arguments
            .into_iter()
            .fold(Expression::module(ast.name.span(), id), |function, ast| {
                let span = function.span.union(ast.span());
                function.apply_to(span, Expression::convert(analyzer, ast))
            })
    }

    fn convert_for_statement(analyzer: &mut Analyzer, ast: syntax::ForStatement) -> Self {
        let else_block = ast
            .else_block
            .map(|ast| Expression::convert_block(analyzer, ast));

        else_block
            .into_iter()
            .chain(ast.branches.into_iter().rev().map(|branch| {
                let for_span = branch.for_token().span;
                let span = branch.span();
                analyzer.push_scope();
                let query = Expression::convert_query(analyzer, branch.query);
                let value = Expression::convert_block(analyzer, branch.body);
                analyzer.pop_scope();
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
        analyzer: &mut Analyzer,
        query: syntax::Query,
        expression: syntax::Expression,
    ) -> Self {
        let span = query.span().union(expression.span());
        analyzer.push_scope();
        let query = Self::convert_query(analyzer, query);
        let body = Self::convert(analyzer, expression);
        analyzer.pop_scope();
        Self::iterator(span, query, body)
    }

    fn convert_template(analyzer: &mut Analyzer, ast: syntax::Template) -> Self {
        let span = ast.span();
        let prefix = Self::string(ast.prefix_token().span, ast.prefix());
        match ast.tag {
            Some(tag) => {
                let (strings, interpolations) = ast
                    .segments
                    .into_iter()
                    .map(|seg| {
                        let suffix = Self::string(seg.suffix_token().span, seg.suffix());
                        let interpolation = Self::convert(analyzer, seg.interpolation);
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

                let tag = Identifier::declared(analyzer, &tag)
                    .map(|tag| Expression::reference(tag.span, tag))
                    .unwrap_or_else(|| {
                        analyzer.error(Error::UnboundIdentifier { name: tag.clone() });
                        // TODO: is dynamic the best way?
                        Self::dynamic(tag)
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
                        let interpolation = Self::convert(analyzer, seg.interpolation);
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

    pub(super) fn iterator(span: Span, query: Expression, value: Expression) -> Self {
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

    pub(super) fn module(span: Span, id: Identifier) -> Self {
        Self::new(span, Value::Module(Box::new(id)))
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
    Module(Box<Identifier>),
    Reference(Box<Identifier>),
    Dynamic(Box<syntax::Identifier>),
    Assert(Box<Assert>),
    End,
}

impl Value {
    pub fn bindings(&self) -> Box<dyn std::iter::Iterator<Item = Id> + '_> {
        match self {
            Self::Sequence(seq) => Box::new(seq.iter().flat_map(|expr| expr.bindings())),
            Self::Pack(pack) => Box::new(pack.bindings()),
            Self::Mapping(pair) => Box::new(pair.0.bindings().chain(pair.1.bindings())),
            Self::Conjunction(pair) => Box::new(pair.0.bindings().chain(pair.1.bindings())),
            Self::Disjunction(pair) => Box::new(pair.0.bindings().chain(pair.1.bindings())),
            // Self::Query(query) => Box::new(query.bindings()),
            Self::Application(application) => Box::new(application.bindings()),
            Self::Reference(ident) => Box::new(std::iter::once(ident.id.clone())),
            _ => Box::new(std::iter::empty()),
        }
    }
}
