use trilogy_scanner::{Token, TokenType};

pub(crate) trait TokenPattern {
    fn matches(&self, token: &Token) -> bool;
}

impl TokenPattern for TokenType {
    fn matches(&self, token: &Token) -> bool {
        token.token_type == *self
    }
}

impl<const N: usize> TokenPattern for [TokenType; N] {
    fn matches(&self, token: &Token) -> bool {
        self.iter()
            .any(|token_type| token.token_type == *token_type)
    }
}
