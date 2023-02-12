use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub enum MutModifier {
    Not,
    Mut(Token),
}

impl MutModifier {
    pub(crate) fn parse(parser: &mut Parser) -> Self {
        parser
            .expect(KwMut)
            .map(MutModifier::Mut)
            .unwrap_or(MutModifier::Not)
    }
}
