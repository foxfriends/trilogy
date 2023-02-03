use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub enum Unification {
    Direct(Box<DirectUnification>),
    Element(Box<ElementUnification>),
    Boolean(Box<BooleanUnification>),
    Not(Box<NotUnification>),
    Parenthesized(Box<ParenthesizedQuery>),
    Lookup(Box<Lookup>),
    True(Box<Token>),
    False(Box<Token>),
}

#[derive(Clone, Debug)]
pub struct DirectUnification {
    pub pattern: Pattern,
    pub expression: Expression,
}

#[derive(Clone, Debug)]
pub struct ElementUnification {
    pub pattern: Pattern,
    pub expression: Expression,
}

#[derive(Clone, Debug)]
pub struct BooleanUnification {
    start: Token,
    pub expression: Expression,
}

#[derive(Clone, Debug)]
pub struct NotUnification {
    start: Token,
    pub query: Unification,
}

#[derive(Clone, Debug)]
pub struct ParenthesizedQuery {
    start: Token,
    pub query: Query,
    end: Token,
}
