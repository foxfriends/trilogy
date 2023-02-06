use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned)]
pub struct KeywordReference {
    start: Token,
    pub keyword: Keyword,
    end: Token,
}

#[derive(Clone, Debug, Spanned)]
pub enum Keyword {
    And(Token),
    Or(Token),
    Add(Token),
    Subtract(Token),
    Multiply(Token),
    Divide(Token),
    Remainder(Token),
    Power(Token),
    IntDivide(Token),
    StructuralEquality(Token),
    ReferenceEquality(Token),
    Lt(Token),
    Gt(Token),
    Leq(Token),
    Geq(Token),
    BitwiseAnd(Token),
    BitwiseOr(Token),
    BitwiseXor(Token),
    LeftShift(Token),
    RightShift(Token),
    Sequence(Token),
    Cons(Token),
    Glue(Token),
    Compose(Token),
    RCompose(Token),
    Pipe(Token),
    RPipe(Token),
    Not(Token),
    Invert(Token),
    Yield(Token),
    Resume(Token),
    Cancel(Token),
    Return(Token),
    Break(Token),
    Continue(Token),
}
