use crate::syntax::{Amble, Document, SyntaxError};
use crate::{Parse, Spanned, TokenPattern};
use peekmore::{PeekMore, PeekMoreIterator};
use trilogy_scanner::{Token, TokenType};

/// The parser for the Trilogy Programming Language.
///
/// This parser takes a sequence of [`Token`][]s, typically from a [`Scanner`][trilogy_scanner::Scanner],
/// and constructs it into an AST, which we call a [`Document`][].
pub struct Parser<'src> {
    source: PeekMoreIterator<Box<dyn Iterator<Item = Token> + 'src>>,
    warnings: Vec<SyntaxError>,
    #[cfg(test)] // expose this thing to the test framework only
    pub(crate) errors: Vec<SyntaxError>,
    #[cfg(not(test))]
    errors: Vec<SyntaxError>,
    pub(crate) is_line_start: bool,
    pub(crate) is_spaced: bool,
}

impl<'src> Parser<'src> {
    /// Construct a new parser taking input from an iterator of [`Token`][]s. The usual
    /// choice is to use a [`Scanner`][trilogy_scanner::Scanner]
    pub fn new<S: Iterator<Item = Token> + 'src>(source: S) -> Self {
        Self {
            source: (Box::new(source) as Box<dyn Iterator<Item = Token>>).peekmore(),
            errors: vec![],
            warnings: vec![],
            is_line_start: true,
            is_spaced: false,
        }
    }

    /// Consume the tokens provided, attempting to build a [`Document`][] from them.
    ///
    /// Where possible, errors are recovered from and collected for later. The returned
    /// `Document` may not be used if the [`Parse`][] contains errors
    pub fn parse(mut self) -> Parse<Document> {
        let ast = Amble::<Document>::parse(&mut self);
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

    pub(crate) fn chomp(&mut self) {
        let mut invalid_tokens = vec![];
        loop {
            let token = self.source.peek().expect("Peeked too many tokens");
            if [
                TokenType::EndOfLine,
                TokenType::CommentBlock,
                TokenType::CommentLine,
                TokenType::CommentInline,
                TokenType::Space,
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

    fn peek_chomp(&mut self) {
        loop {
            let Some(token) = self.source.peek() else {
                return;
            };
            if token.token_type == TokenType::EndOfFile {
                return;
            }
            if [
                TokenType::EndOfLine,
                TokenType::CommentBlock,
                TokenType::CommentLine,
                TokenType::CommentInline,
                TokenType::Space,
                TokenType::Error,
            ]
            .matches(token)
            {
                self.source.advance_cursor();
                continue;
            }
            break;
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
            self.is_line_start = [EndOfLine, CommentLine, DocInner, DocOuter, ByteOrderMark, StartOfFile].matches(&token) || self.is_line_start && [CommentInline, Space].matches(&token);
            self.is_spaced = [EndOfLine, CommentLine, DocInner, DocOuter, CommentInline, CommentBlock, Space].matches(&token);
        };
        token
    }

    fn peek_next(&mut self) -> Option<Token> {
        // Technically probably shouldn't unwrap here but if we consume the EndOfFile
        // it has to be at the end, at which point we consume no more, so this should
        // be safe.
        self.peek_chomp();
        let peeked = self.source.peek().cloned();
        self.source.advance_cursor();
        peeked
    }

    pub(crate) fn expect_bang_oparen(&mut self) -> Result<(Token, Token), Token> {
        use TokenType::*;
        // Though tokenized as two tokens, this is kind of treated as one token in some cases,
        // requiring `!(` to be unspaced in procedure calls. Since whitespace is a token, this
        // a low-level peekmore after the high-level peek will sufficiently detect this.
        let next = self.peek().clone();
        let after = self.source.peek_nth(1);
        if next.token_type == OpBang && after.unwrap().token_type == OParen {
            let bang = self.expect(OpBang).unwrap();
            let oparen = self.expect(OParen).unwrap();
            Ok((bang, oparen))
        } else {
            Err(next)
        }
    }

    pub(crate) fn peek(&mut self) -> &Token {
        self.chomp();
        self.source.peek().unwrap()
    }

    pub(crate) fn force_peek(&mut self) -> &Token {
        self.source.peek().unwrap()
    }

    pub(crate) fn peekn(&mut self, n: usize) -> Option<Vec<Token>> {
        self.chomp();
        let tokens = (0..n).map(|_| self.peek_next()).collect::<Option<Vec<_>>>();
        self.source.reset_cursor();
        tokens
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

    pub(crate) fn consume(&mut self) -> Token {
        self.chomp();
        self.next()
    }

    pub(crate) fn check(&mut self, pattern: impl TokenPattern) -> Result<&Token, &Token> {
        let token = self.peek();
        if pattern.matches(token) {
            Ok(token)
        } else {
            Err(token)
        }
    }

    pub(crate) fn predict(&mut self, pattern: impl TokenPattern) -> bool {
        self.peek_next();
        let second = self.peek_next();
        let result = second
            .as_ref()
            .map(|token| pattern.matches(token))
            .unwrap_or(false);
        self.source.reset_cursor();
        result
    }
}
