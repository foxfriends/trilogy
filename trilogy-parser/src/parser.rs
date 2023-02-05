use crate::syntax::{Document, SyntaxError};
use crate::Spanned;
use std::iter::Peekable;
use trilogy_scanner::{Scanner, Token, TokenType};

pub struct Parser<'src> {
    source: Peekable<Scanner<'src>>,
    warnings: Vec<SyntaxError>,
    errors: Vec<SyntaxError>,
    discarded_tokens: Vec<Token>,
    is_line_ended: bool,
    is_line_start: bool,
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
            discarded_tokens: vec![],
            is_line_ended: true,
            is_line_start: true,
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

    fn next(&mut self) -> Token {
        // Technically probably shouldn't unwrap here but if we consume the EndOfFile
        // it has to be at the end, at which point we consume no more, so this should
        // be safe.
        let token = self.source.next().expect("Consumed too many times");
        #[rustfmt::skip] {
            use TokenType::*;
            // Different types of whitespace imply that we are truly at the start of a line
            // without any leading (non-whitespace) characters, as opposed to only the first
            // whole token on a line but other partial tokens were on this line already
            // (specifically, block comments).
            //
            // The ByteOrderMark, while not technically whitespace (or even allowed in most
            // parts of the code, for that matter) is included here because its presence is
            // not considered at all, so should not change the initial states of these bits
            // in much the same way that StartOfFile does not change them.
            //
            // That said, cases where line endings and startings are needed are uncertain,
            // maybe I don't need both of these flags.
            self.is_line_ended = [EndOfLine, CommentLine, DocInner, DocOuter, CommentBlock, ByteOrderMark, StartOfFile].matches(&token);
            self.is_line_start = [EndOfLine, CommentLine, DocInner, DocOuter, ByteOrderMark, StartOfFile].matches(&token);
        };
        token
    }

    pub(crate) fn consume(&mut self) -> Token {
        self.flush();
        self.next()
    }

    fn flush(&mut self) -> Option<SyntaxError> {
        if self.discarded_tokens.is_empty() {
            return None;
        }
        Some(self.error(SyntaxError::new(
            self.discarded_tokens.span(),
            "Unexpected tokens in input.".to_owned(),
        )))
    }

    pub(crate) fn discard(&mut self) {
        let token = self.next();
        self.discarded_tokens.push(token);
    }

    pub(crate) fn peek(&mut self) -> &Token {
        // Same logic as the consume... If we peek EndOfFile, don't consume it, then
        // this will be ok.
        self.source
            .peek()
            .expect("Peeked after consuming too many times")
    }

    pub(crate) fn expect(&mut self, pattern: impl TokenPattern) -> Option<Token> {
        let token = self.peek();
        if pattern.matches(token) {
            Some(self.consume())
        } else {
            None
        }
    }

    pub(crate) fn chomp(&mut self) {
        while self
            .check([
                TokenType::EndOfLine,
                TokenType::CommentBlock,
                TokenType::CommentLine,
                TokenType::CommentInline,
            ])
            .is_some()
        {
            // Chomping whitespace does not typically change the panic state,
            // so don't consume here.
            self.next();
        }
    }

    pub(crate) fn check(&mut self, pattern: impl TokenPattern) -> Option<&Token> {
        let token = self.peek();
        if pattern.matches(token) {
            Some(token)
        } else {
            None
        }
    }

    pub(crate) fn is_line_start(&self) -> bool {
        self.is_line_start
    }

    #[allow(dead_code)]
    pub(crate) fn is_line_ended(&self) -> bool {
        self.is_line_ended
    }
}
