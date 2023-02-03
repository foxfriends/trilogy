use trilogy_scanner::Token;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct CharacterLiteral {
    token: Token,
}
