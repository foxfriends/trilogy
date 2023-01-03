use super::token::Token;
use crate::TokenType::{self, *};
use source_span::{DefaultMetrics, Span};
use std::iter::Peekable;
use std::str::Chars;
use std::string::String;

const METRICS: DefaultMetrics = DefaultMetrics::with_tab_stop(4);

#[derive(Clone, Debug)]
pub struct Scanner<'a> {
    chars: Peekable<Chars<'a>>,
    span: Span,
    is_started: bool,
    is_finished: bool,
    nesting: Vec<char>,
}

trait CharPattern {
    fn check(&self, ch: char) -> bool;
}

impl CharPattern for char {
    fn check(&self, ch: char) -> bool {
        *self == ch
    }
}

impl<F> CharPattern for F
where
    F: Fn(char) -> bool,
{
    fn check(&self, ch: char) -> bool {
        self(ch)
    }
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().peekable(),
            span: Span::default(),
            is_started: false,
            is_finished: false,
            nesting: vec![],
        }
    }

    fn make_token(&mut self, token_type: TokenType) -> Token {
        Token::new(token_type, self.span)
    }

    fn make_error(&mut self, error_message: &'static str) -> Token {
        self.make_token(TokenType::Error).with_value(error_message)
    }

    fn context(&self, ch: char) -> bool {
        self.nesting.last().map(|nest| *nest == ch).unwrap_or(false)
    }

    fn consume(&mut self) -> Option<char> {
        let ch = self.chars.next()?;
        self.span.push(ch, &METRICS);
        Some(ch)
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn expect<P: CharPattern>(&mut self, pattern: P) -> Option<char> {
        let ch = self.peek()?;
        if !pattern.check(ch) {
            return None;
        }
        self.consume()
    }

    fn identifier(&mut self, mut value: String) -> String {
        while self
            .chars
            .peek()
            .copied()
            .map(is_identifier)
            .unwrap_or(false)
        {
            value.push(self.consume().unwrap());
        }
        value
    }

    fn identifier_or_keyword(&mut self, starts_with: char) -> Token {
        let mut identifier = self.identifier(starts_with.into());
        if self.expect('!').is_some() {
            identifier.push('!');
        }
        self.make_token(Identifier)
            .with_value(identifier)
            .resolve_keywords()
    }

    fn unicode_escape_sequence(&mut self) -> Option<char> {
        self.expect('{')?;
        let a = self.expect(|ch: char| ch.is_ascii_alphanumeric())?;
        let b = self.expect(|ch: char| ch.is_ascii_alphanumeric())?;
        let c = self.expect(|ch: char| ch.is_ascii_alphanumeric())?;
        let d = self.expect(|ch: char| ch.is_ascii_alphanumeric())?;
        self.expect('}')?;
        char::from_u32(
            hex_to_u32(a) << 12 & hex_to_u32(b) << 8 & hex_to_u32(c) << 4 & hex_to_u32(d),
        )
    }

    fn ascii_escape_sequence(&mut self) -> Option<char> {
        let a = self.expect(|ch: char| ch.is_ascii_alphanumeric())?;
        let b = self.expect(|ch: char| ch.is_ascii_alphanumeric())?;
        char::from_u32(hex_to_u32(a) << 4 & hex_to_u32(b))
    }

    fn escape_sequence(&mut self) -> Result<char, &'static str> {
        self.consume()
            .ok_or("Unexpected end of file in escape sequence")
            .and_then(|ch| match ch {
                'u' => self
                    .unicode_escape_sequence()
                    .ok_or("Invalid Unicode escape sequence"),
                'x' => self
                    .ascii_escape_sequence()
                    .ok_or("invalid ASCII escape sequence"),
                '"' | '\'' | '$' | '\\' => Ok(ch),
                'n' => Ok('\n'),
                't' => Ok('\t'),
                'r' => Ok('\r'),
                '0' => Ok('\0'),
                _ => Err("Invalid escape sequence."),
            })
    }

    fn char_escape(&mut self) -> Token {
        let ch = match self.escape_sequence() {
            Ok(ch) => ch,
            Err(message) => return self.make_error(message),
        };
        if self.expect('\'').is_none() {
            return self.make_error("A character literal may represent only a single character.");
        }
        self.make_token(Character).with_value(ch)
    }

    fn char_or_atom(&mut self) -> Token {
        if self.expect('\\').is_some() {
            return self.char_escape();
        }
        let Some(ch) = self.consume() else {
            return self.make_error("Unexpected end of file in character literal");
        };
        if self.expect('\'').is_some() {
            return self.make_token(Character).with_value(ch);
        }
        self.make_token(Atom).with_value(self.identifier(ch.into()))
    }

    fn block_comment(&mut self) -> Token {
        let mut contents = String::new();
        let mut is_inline = true;
        let mut depth = 1;
        while let Some(ch) = self.consume() {
            if ch == '\n' {
                is_inline = false;
            }
            match ch {
                '-' if self.expect('#').is_some() => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    contents.push_str("-#"); // correction
                }
                '#' if self.expect('-').is_some() => {
                    depth += 1;
                    if depth == 0 {
                        break;
                    }
                    contents.push_str("#-"); // correction
                }
                _ => contents.push(ch),
            }
        }
        self.make_token(if is_inline {
            CommentInline
        } else {
            CommentBlock
        })
        .with_value(contents)
    }

    fn finish_line(&mut self) -> String {
        let mut line = String::new();
        while let Some(ch) = self.consume() {
            line.push(ch);
            if ch == '\n' {
                break;
            }
        }
        line
    }

    fn comment(&mut self) -> Token {
        if self.expect('-').is_some() {
            return self.block_comment();
        }
        if self.expect('#').is_some() {
            return self.make_token(DocOuter).with_value(self.finish_line());
        }
        if self.expect('!').is_some() {
            return self.make_token(DocInner).with_value(self.finish_line());
        }
        self.make_token(CommentLine).with_value(self.finish_line())
    }

    // Not my best method signature, but it gets the job done.
    fn string_or_template(&mut self, allow_templates: bool, on_end: TokenType) -> Token {
        let mut content = String::new();
        while let Some(mut ch) = self.consume() {
            if ch == '"' {
                return self.make_token(on_end).with_value(content);
            }
            if allow_templates && ch == '$' && self.expect(|ch| ch == '{').is_some() {
                return self.make_token(TemplateStart).with_value(content);
            }
            if ch == '\\' {
                ch = match self.escape_sequence() {
                    Ok(ch) => ch,
                    Err(message) => {
                        return self.make_error(message);
                    }
                }
            }
            content.push(ch)
        }
        self.make_error("Unexpected end of file found before end of string literal.")
    }
}

impl Iterator for Scanner<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        // Special cases to insert the start and end tokens once each
        if !self.is_started {
            self.is_started = true;
            return Some(self.make_token(StartOfFile));
        }
        if self.is_finished {
            return None;
        }
        if self.peek().is_none() {
            self.is_finished = true;
            return Some(self.make_token(EndOfFile));
        }
        // Skip whitespaces in between tokens when there was more than one in a row
        while self.peek().map(is_ignored_whitespace).unwrap_or(false) {
            self.consume();
            self.span.clear();
        }

        // Match the token. We do some weird stuff with nesting here,
        // as the interpretation of a `}` character is pretty different
        // depending on whether it is within a template string or not.
        //
        // No errors are reported about invalid nesting, that should be
        // handled by a later pass.
        //
        // Might be interesting to see how the Javascript parser handles
        // it, but I kinda suspect it is similar...
        let token = match self.consume().unwrap() {
            '\u{FEFF}' => self.make_token(ByteOrderMark),
            ch @ ('_' | 'a'..='z' | 'A'..='Z') => self.identifier_or_keyword(ch),
            '\'' => self.char_or_atom(),
            '#' => self.comment(),
            '"' => self.string_or_template(false, String),
            // Feels slightly irresponsible to put side effects into a guard...
            '$' if self.expect(|ch| ch == '"').is_some() => {
                self.consume();
                self.string_or_template(true, String)
            }
            '{' => {
                self.nesting.push('{');
                self.make_token(OBrace)
            }
            '}' if self.context('$') => self.string_or_template(true, TemplateEnd),
            '}' => {
                if self.context('{') {
                    self.nesting.pop();
                }
                self.make_token(CBrace)
            }
            '(' => {
                self.nesting.push('(');
                self.make_token(OParen)
            }
            ')' => {
                if self.context('(') {
                    self.nesting.pop();
                }
                self.make_token(CParen)
            }
            '[' => {
                self.nesting.push('[');
                self.make_token(OBrack)
            }
            ']' => {
                if self.context('[') {
                    self.nesting.pop();
                }
                self.make_token(CParen)
            }
            '\n' => self.make_token(EndOfLine),
            // '0'..='9' => self.numeric(),
            // operators...
            _ => todo!("Working on it..."),
        };
        if token.token_type == TemplateEnd && self.context('$') {
            self.nesting.pop();
        }

        self.span.clear();
        Some(token)
    }
}

fn is_identifier(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn is_ignored_whitespace(ch: char) -> bool {
    // \n is not ignored whitespace, though it is valid.
    // Other whitespace is not valid except in strings.
    ch == '\r' || ch == '\t' || ch == ' '
}

fn hex_to_u32(ch: char) -> u32 {
    match ch {
        '0' => 0,
        '1' => 1,
        '2' => 2,
        '3' => 3,
        '4' => 4,
        '5' => 5,
        '6' => 6,
        '7' => 7,
        '8' => 8,
        '9' => 9,
        'A' | 'a' => 10,
        'B' | 'b' => 11,
        'C' | 'c' => 12,
        'D' | 'd' => 13,
        'E' | 'e' => 14,
        'F' | 'f' => 15,
        _ => panic!("Invalid character for hexadecimal value"),
    }
}
