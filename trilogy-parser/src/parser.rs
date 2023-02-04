use crate::syntax::{Document, SyntaxError};
use std::iter::Peekable;
use trilogy_scanner::{Scanner, Token, TokenType};

pub struct Parser<'src> {
    source: Peekable<Scanner<'src>>,
    warnings: Vec<SyntaxError>,
    errors: Vec<SyntaxError>,
}

pub struct ParseResult {
    pub ast: Document,
    pub warnings: Vec<SyntaxError>,
    pub errors: Vec<SyntaxError>,
}

pub(crate) trait TokenPattern {
    fn matches(self, token: &Token) -> bool;
}

impl TokenPattern for TokenType {
    fn matches(self, token: &Token) -> bool {
        token.token_type == self
    }
}

impl<const N: usize> TokenPattern for [TokenType; N] {
    fn matches(self, token: &Token) -> bool {
        self.into_iter()
            .any(|token_type| token.token_type == token_type)
    }
}

impl<'src> Parser<'src> {
    pub fn new(source: Scanner<'src>) -> Self {
        Self {
            source: source.peekable(),
            errors: vec![],
            warnings: vec![],
        }
    }

    pub fn parse(mut self) -> ParseResult {
        let ast = Document::parse(&mut self);
        ParseResult {
            ast,
            warnings: self.warnings,
            errors: self.errors,
        }
    }
}

impl Parser<'_> {
    #[cfg_attr(not(feature = "lax"), allow(dead_code))]
    pub(crate) fn warn(&mut self, warning: SyntaxError) {
        self.warnings.push(warning);
    }

    #[cfg_attr(feature = "lax", allow(dead_code))]
    pub(crate) fn error(&mut self, error: SyntaxError) -> SyntaxError {
        self.errors.push(error.clone());
        error
    }

    pub(crate) fn consume(&mut self) -> Option<Token> {
        self.source.next()
    }

    pub(crate) fn peek(&mut self) -> Option<&Token> {
        self.source.peek()
    }

    pub(crate) fn expect(&mut self, pattern: impl TokenPattern) -> Option<Token> {
        let token = self.peek()?;
        if pattern.matches(token) {
            self.consume()
        } else {
            None
        }
    }

    pub(crate) fn chomp(&mut self) {
        while self
            .expect([
                TokenType::EndOfLine,
                TokenType::CommentBlock,
                TokenType::CommentLine,
                TokenType::CommentInline,
            ])
            .is_some()
        {}
    }

    pub(crate) fn check(&mut self, pattern: impl TokenPattern) -> Option<&Token> {
        let token = self.peek()?;
        if pattern.matches(token) {
            Some(token)
        } else {
            None
        }
    }
}
