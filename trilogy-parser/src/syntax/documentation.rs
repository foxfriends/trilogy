use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug)]
pub struct Documentation {
    tokens: Vec<Token>,
}

impl Documentation {
    pub(crate) fn parse_inner(parser: &mut Parser) -> Option<Self> {
        let mut tokens = vec![];

        loop {
            parser.chomp();
            let Some(token) = parser.expect(DocInner) else {
                break;
            };
            tokens.push(token);
        }

        if tokens.is_empty() {
            return None;
        }

        Some(Self { tokens })
    }
}
