use crate::syntax::{Document, SyntaxError};
use crate::{Spanned, TokenPattern};
use std::iter::Peekable;
use trilogy_scanner::{Scanner, Token, TokenType};

pub struct Parser<'src> {
    source: Peekable<Scanner<'src>>,
    warnings: Vec<SyntaxError>,
    errors: Vec<SyntaxError>,
    is_line_ended: bool,
    is_line_start: bool,
}

pub struct Parse {
    pub ast: Document,
    pub warnings: Vec<SyntaxError>,
    pub errors: Vec<SyntaxError>,
}

impl<'src> Parser<'src> {
    pub fn new(source: Scanner<'src>) -> Self {
        Self {
            source: source.peekable(),
            errors: vec![],
            warnings: vec![],
            is_line_ended: true,
            is_line_start: true,
        }
    }

    pub fn parse(mut self) -> Parse {
        let ast = Document::parse(&mut self);
        Parse {
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

    pub(crate) fn error(&mut self, error: SyntaxError) {
        self.errors.push(error);
    }

    pub(crate) fn expected(
        &mut self,
        token: Token,
        message: impl std::fmt::Display,
    ) -> SyntaxError {
        let error = SyntaxError::new(token.span, message);
        self.error(error.clone());
        error
    }

    fn chomp(&mut self) {
        let mut invalid_tokens = vec![];
        loop {
            let token = self.source.peek().expect("Peeked too many tokens");
            if [
                TokenType::EndOfLine,
                TokenType::CommentBlock,
                TokenType::CommentLine,
                TokenType::CommentInline,
            ]
            .matches(token)
            {
                self.next();
                continue;
            }
            if TokenType::Error.matches(token) {
                invalid_tokens.push(self.next());
                continue;
            }
            break;
        }
        if !invalid_tokens.is_empty() {
            self.error(SyntaxError::new(
                invalid_tokens.span(),
                "invalid characters in input",
            ));
        }
    }

    fn next(&mut self) -> Token {
        // Technically probably shouldn't unwrap here but if we consume the EndOfFile
        // it has to be at the end, at which point we consume no more, so this should
        // be safe.
        let token = self.source.next().expect("Consumed too many tokens");

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

    pub(crate) fn peek(&mut self) -> &Token {
        self.chomp();
        self.source.peek().unwrap()
    }

    pub(crate) fn synchronize(&mut self, pattern: impl TokenPattern) {
        while !pattern.matches(self.peek()) {
            self.next();
        }
    }

    pub(crate) fn expect(&mut self, pattern: impl TokenPattern) -> Result<Token, Token> {
        let token = self.peek();
        if !pattern.matches(token) {
            return Err(token.clone());
        }
        Ok(self.next())
    }

    #[allow(dead_code)]
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
