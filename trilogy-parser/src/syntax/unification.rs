use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub enum Unification {
    Direct(Box<DirectUnification>),
    Element(Box<ElementUnification>),
    Boolean(Box<BooleanUnification>),
    Not(Box<NotUnification>),
    Parenthesized(Box<ParenthesizedQuery>),
    Lookup(Box<Lookup>),
    True(Box<Token>),
    False(Box<Token>),
    SyntaxError(Box<SyntaxError>),
}
