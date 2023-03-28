use super::*;
use crate::{Analyzer, Error};
use source_span::Span;
use trilogy_parser::{syntax, Spanned};

#[derive(Clone, Debug)]
pub struct Expression {
    pub span: Span,
    pub value: Value,
}

impl Expression {
    pub(super) fn convert(_analyzer: &mut Analyzer, _ast: syntax::Expression) -> Self {
        todo!()
    }

    pub(super) fn convert_block(analyzer: &mut Analyzer, ast: syntax::Block) -> Self {
        let span = ast.span();
        let sequence = Self::convert_sequence(analyzer, &mut ast.statements.into_iter());
        Self::sequence(span, sequence)
    }

    fn convert_sequence(
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
            Let(ast) => {
                let span = ast.span();
                let query = Query::convert(analyzer, ast.query);
                analyzer.push_scope();
                let body = Self::convert_sequence(analyzer, rest);
                analyzer.pop_scope();
                // TODO: Span::default() is not best here, but there's not really a proper span for
                // this, so what to do?
                Self::r#let(span, query, Self::sequence(Span::default(), body))
            }
            Assignment(..) => todo!(),
            FunctionAssignment(..) => todo!(),
            If(..) => todo!(),
            Match(..) => todo!(),
            While(..) => todo!(),
            For(..) => todo!(),
            Break(..) => todo!(),
            Continue(..) => todo!(),
            Resume(..) => todo!(),
            Cancel(..) => todo!(),
            Return(..) => todo!(),
            End(..) => todo!(),
            Exit(..) => todo!(),
            Yield(..) => todo!(),
            Expression(ast) => Self::convert(analyzer, *ast),
            Assert(..) => todo!(),
            Handled(..) => todo!(),
            Block(ast) => {
                analyzer.push_scope();
                let block = Self::convert_block(analyzer, *ast);
                analyzer.pop_scope();
                block
            }
        }
    }

    pub(super) fn convert_query(analyzer: &mut Analyzer, ast: syntax::Query) -> Self {
        let span = ast.span();
        let query = Query::convert(analyzer, ast);
        Self::query(span, query)
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

    pub(super) fn r#let(span: Span, query: Query, body: Expression) -> Self {
        Self {
            span,
            value: Value::Let(Box::new(Let::new(query, body))),
        }
    }

    pub(super) fn dynamic(identifier: syntax::Identifier) -> Self {
        Self {
            span: identifier.span(),
            value: Value::Dynamic(Box::new(identifier)),
        }
    }

    pub(super) fn module(span: Span, id: Identifier) -> Self {
        Self {
            span,
            value: Value::Module(Box::new(id)),
        }
    }

    pub(super) fn sequence(span: Span, sequence: Vec<Expression>) -> Self {
        Self {
            span,
            value: Value::Sequence(sequence),
        }
    }

    pub(super) fn query(span: Span, query: Query) -> Self {
        Self {
            span,
            value: Value::Query(Box::new(query)),
        }
    }

    pub(super) fn application(span: Span, lhs: Expression, rhs: Expression) -> Self {
        Self {
            span,
            value: Value::Application(Box::new(Application::new(lhs, rhs))),
        }
    }

    pub(super) fn apply_to(self, span: Span, rhs: Expression) -> Self {
        Self::application(span, self, rhs)
    }

    pub(super) fn builtin(span: Span, builtin: Builtin) -> Self {
        Self {
            span,
            value: Value::Builtin(builtin),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    Builtin(Builtin),
    Pack(Box<Pack>),
    Sequence(Vec<Expression>),
    Mapping(Box<(Expression, Expression)>),
    Number(Box<NumberLiteral>),
    Character(Box<CharacterLiteral>),
    String(Box<StringLiteral>),
    Bits(Box<BitsLiteral>),
    Boolean(Box<BooleanLiteral>),
    Unit(Box<UnitLiteral>),
    Atom(Box<AtomLiteral>),
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
}
