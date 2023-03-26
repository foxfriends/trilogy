use super::*;
use crate::Analyzer;
use source_span::Span;
use trilogy_parser::{syntax, Spanned};

#[derive(Clone, Debug)]
pub struct Pattern {
    span: Span,
    pub value: Value,
}

impl Pattern {
    pub(super) fn convert(_analyzer: &mut Analyzer, _ast: syntax::Pattern) -> Self {
        todo!()
    }

    pub(super) fn binding(analyzer: &mut Analyzer, ast: syntax::Identifier) -> Self {
        let span = ast.span();
        let id = Identifier::declare(analyzer, ast);
        Self {
            span,
            value: Value::Binding(Box::new(id)),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    Conjunction(Box<(Pattern, Pattern)>),
    Disjunction(Box<(Pattern, Pattern)>),
    Mapping(Box<(Pattern, Pattern)>),
    Number(Box<NumberLiteral>),
    Character(Box<CharacterLiteral>),
    String(Box<StringLiteral>),
    Bits(Box<BitsLiteral>),
    Boolean(Box<BooleanLiteral>),
    Unit(Box<UnitLiteral>),
    Atom(Box<AtomLiteral>),
    Negative(Box<Pattern>),
    Glue(Box<GluePattern>),
    Struct(Box<StructPattern>),
    Tuple(Box<(Pattern, Pattern)>),
    Array(Box<ArrayPattern>),
    Set(Box<SetPattern>),
    Record(Box<RecordPattern>),
    Pinned(Box<Identifier>),
    Binding(Box<Identifier>),
    Wildcard,
}
