use trilogy_scanner::Token;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct UnitLiteral {
    token: Token,
}
