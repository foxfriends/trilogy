use super::token::Token;
use crate::TokenType::{self, *};
use bitvec::prelude::*;
use num::{BigInt, BigRational, Complex, Num, Zero};
use peekmore::{PeekMore, PeekMoreIterator};
use source_span::{DefaultMetrics, Span};
use std::ops::{Range, RangeInclusive};
use std::str::Chars;
use std::string::String;

const METRICS: DefaultMetrics = DefaultMetrics::with_tab_stop(4);

/// The scanner (lexer) for the Trilogy Programming Language.
///
/// The `Scanner` is expected to be used as an [`Iterator`][] over
/// [`Token`][]s.
#[derive(Clone, Debug)]
pub struct Scanner<'a> {
    chars: PeekMoreIterator<Chars<'a>>,
    span: Span,
    is_started: bool,
    is_first_character: bool,
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

impl CharPattern for Range<char> {
    fn check(&self, ch: char) -> bool {
        self.contains(&ch)
    }
}

impl CharPattern for RangeInclusive<char> {
    fn check(&self, ch: char) -> bool {
        self.contains(&ch)
    }
}

impl CharPattern for &'static str {
    fn check(&self, ch: char) -> bool {
        self.contains(ch)
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

enum Numberlike {
    Complete(BigInt),
    Incomplete(BigInt),
    Bits(BitVec<usize, Msb0>),
}

enum BitsOrNumber<T> {
    Number(T),
    Bits(BitVec<usize, Msb0>),
}

impl<'a> Scanner<'a> {
    /// Create a new scanner that scans the provided source string.
    ///
    /// Consume this scanner as an [`Iterator`][].
    #[must_use]
    pub fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().peekmore(),
            span: Span::default(),
            is_started: false,
            is_first_character: true,
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
        self.nesting.last().is_some_and(|nest| *nest == ch)
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

    fn predict<P: CharPattern>(&mut self, pattern: P) -> bool {
        self.chars
            .peek_nth(1)
            .copied()
            .is_some_and(|ch| pattern.check(ch))
    }

    fn identifier(&mut self, mut value: String) -> String {
        while self.chars.peek().copied().is_some_and(is_identifier) {
            value.push(self.consume().unwrap());
        }
        value
    }

    fn identifier_or_keyword(&mut self, starts_with: char) -> Token {
        let identifier = self.identifier(starts_with.into());
        let token = if self.expect('=').is_some() {
            self.make_token(IdentifierEq).with_value(identifier)
        } else {
            self.make_token(Identifier).with_value(identifier)
        };

        token
            .resolve_keywords()
            .unwrap_or_else(|err| self.make_error(err))
    }

    fn unicode_escape_sequence(&mut self) -> Option<char> {
        self.expect('{')?;
        let mut repr = String::new();
        loop {
            if self.expect('}').is_some() {
                break;
            }
            repr.push(self.expect(|ch: char| ch.is_ascii_hexdigit())?);
        }
        let num = u32::from_str_radix(&repr, 16).ok()?;
        char::from_u32(num)
    }

    fn ascii_escape_sequence(&mut self) -> Option<char> {
        let a = self.expect(|ch: char| ch.is_ascii_hexdigit())?;
        let b = self.expect(|ch: char| ch.is_ascii_hexdigit())?;
        char::from_u32((hex_to_u32(a) << 4) & hex_to_u32(b))
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
        if !is_identifier(ch) {
            return self.make_error("Invalid character in atom literal");
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
        match self.expect("#-!") {
            Some('-') => self.block_comment(),
            Some('#') => self.make_token(DocOuter).with_value(self.finish_line()),
            Some('!') => self.make_token(DocInner).with_value(self.finish_line()),
            None => self.make_token(CommentLine).with_value(self.finish_line()),
            _ => unreachable!(),
        }
    }

    fn string_or_template(&mut self, on_continue: TokenType, on_end: TokenType) -> Token {
        let mut content = String::new();
        while let Some(mut ch) = self.consume() {
            if ch == '"' {
                return self.make_token(on_end).with_value(content);
            }
            if ch == '$' && self.expect(|ch| ch == '{').is_some() {
                return self.make_token(on_continue).with_value(content);
            }
            if ch == '\\' {
                ch = match self.escape_sequence() {
                    Ok(ch) => ch,
                    Err(message) => return self.make_error(message),
                };
            }
            content.push(ch);
        }
        self.make_error("Unexpected end of file found before end of string literal.")
    }

    fn fractional(&mut self) -> BigRational {
        let mut value = String::new();
        while let Some(ch) = self.expect('0'..='9') {
            value.push(ch);
            while self.expect('_').is_some() {}
        }
        let num = BigInt::from_str_radix(&value, 10).unwrap();
        if num == BigInt::zero() {
            return BigRational::zero();
        }
        let denom = BigInt::from(10).pow(value.len() as u32);
        BigRational::new(num, denom)
    }

    fn decimal(&mut self, mut value: String) -> BigInt {
        while self.expect('_').is_some() {}
        while let Some(ch) = self.expect('0'..='9') {
            value.push(ch);
            while self.expect('_').is_some() {}
        }
        BigInt::from_str_radix(&value, 10).unwrap()
    }

    fn binary(&mut self) -> BigInt {
        let mut value = String::new();
        while self.expect('_').is_some() {}
        while let Some(ch) = self.expect('0'..='1') {
            value.push(ch);
            while self.expect('_').is_some() {}
        }
        BigInt::from_str_radix(&value, 2).unwrap()
    }

    fn octal(&mut self) -> BigInt {
        let mut value = String::new();
        while self.expect('_').is_some() {}
        while let Some(ch) = self.expect('0'..='7') {
            value.push(ch);
            while self.expect('_').is_some() {}
        }
        BigInt::from_str_radix(&value, 8).unwrap()
    }

    fn hexadecimal(&mut self) -> BigInt {
        let mut value = String::new();
        while self.expect('_').is_some() {}
        while let Some(ch) = self.expect(|ch: char| ch.is_ascii_hexdigit()) {
            value.push(ch);
            while self.expect('_').is_some() {}
        }
        BigInt::from_str_radix(&value, 16).unwrap()
    }

    fn bits_binary(&mut self) -> BitVec<usize, Msb0> {
        let mut value = BitVec::new();
        while self.expect('_').is_some() {}
        while let Some(ch) = self.expect('0'..='1') {
            value.push(ch == '1');
            while self.expect('_').is_some() {}
        }
        value
    }

    fn bits_octal(&mut self) -> BitVec<usize, Msb0> {
        let mut value = BitVec::new();
        while self.expect('_').is_some() {}
        while let Some(ch) = self.expect('0'..='7') {
            value.push(hex_to_u32(ch) & 0b100 > 0);
            value.push(hex_to_u32(ch) & 0b010 > 0);
            value.push(hex_to_u32(ch) & 0b001 > 0);
            while self.expect('_').is_some() {}
        }
        value
    }

    fn bits_hexadecimal(&mut self) -> BitVec<usize, Msb0> {
        let mut value = BitVec::new();
        while self.expect('_').is_some() {}
        while let Some(ch) = self.expect(|ch: char| ch.is_ascii_hexdigit()) {
            value.push(hex_to_u32(ch) & 0b1000 > 0);
            value.push(hex_to_u32(ch) & 0b0100 > 0);
            value.push(hex_to_u32(ch) & 0b0010 > 0);
            value.push(hex_to_u32(ch) & 0b0001 > 0);
            while self.expect('_').is_some() {}
        }
        value
    }

    // Another odd method signature; the bool is whether a floating point
    // literal is to be accepted still.
    //
    // Really though, why do we restrict that? Rationals can represent
    // floating points on non-decimal values too. Maybe a fun little update
    // to the language to implement that if I ever require it.
    fn zero_or_other_base(&mut self) -> Numberlike {
        match self.expect("box") {
            Some('b') if self.expect('b').is_some() => Numberlike::Bits(self.bits_binary()),
            Some('b') if self.expect('o').is_some() => Numberlike::Bits(self.bits_octal()),
            Some('b') if self.expect('x').is_some() => Numberlike::Bits(self.bits_hexadecimal()),
            Some('b') => Numberlike::Complete(self.binary()),
            Some('o') => Numberlike::Complete(self.octal()),
            Some('x') => Numberlike::Complete(self.hexadecimal()),
            None => Numberlike::Incomplete(BigInt::zero()),
            _ => unreachable!(),
        }
    }

    fn integer_or_bits(&mut self, starts_with: char) -> Numberlike {
        match starts_with {
            '0' => self.zero_or_other_base(),
            _ => Numberlike::Incomplete(self.decimal(starts_with.into())),
        }
    }

    fn integer(&mut self, starts_with: char) -> Result<BigInt, Box<Token>> {
        match starts_with {
            '0' => match self.zero_or_other_base() {
                Numberlike::Bits(..) => Err(Box::new(
                    self.make_error("The denominator must be a number, not bits"),
                )),
                Numberlike::Complete(n) | Numberlike::Incomplete(n) => Ok(n),
            },
            _ => Ok(self.decimal(starts_with.into())),
        }
    }

    fn rational_or_float_or_bits(
        &mut self,
        starts_with: char,
    ) -> Result<BitsOrNumber<BigRational>, Box<Token>> {
        let (numerator, can_be_float) = match self.integer_or_bits(starts_with) {
            Numberlike::Complete(number) => (number, false),
            Numberlike::Incomplete(number) => (number, true),
            Numberlike::Bits(bits) => return Ok(BitsOrNumber::Bits(bits)),
        };
        match self.peek() {
            Some('/') if self.predict('0'..='9') => {
                self.expect('/').unwrap();
                let first = self.consume().unwrap();
                let denominator = self.integer(first)?;
                if denominator.is_zero() {
                    return Err(Box::new(
                        self.make_error("The denominator of a rational literal may not be 0"),
                    ));
                }
                Ok(BitsOrNumber::Number(BigRational::new(
                    numerator,
                    denominator,
                )))
            }
            Some('.') if can_be_float && self.predict('0'..='9') => {
                self.expect('.').unwrap();
                let number = self.fractional();
                Ok(BitsOrNumber::Number(BigRational::from(numerator) + number))
            }
            _ => Ok(BitsOrNumber::Number(numerator.into())),
        }
    }

    fn complex_or_bits(
        &mut self,
        starts_with: char,
    ) -> Result<BitsOrNumber<Complex<BigRational>>, Box<Token>> {
        let real = match self.rational_or_float_or_bits(starts_with)? {
            BitsOrNumber::Number(number) => number,
            BitsOrNumber::Bits(bits) => return Ok(BitsOrNumber::Bits(bits)),
        };
        let imaginary = match self.peek() {
            Some('i') if self.predict('0'..='9') => {
                self.consume().unwrap();
                let first = self.consume().unwrap();
                match self.rational_or_float_or_bits(first)? {
                    BitsOrNumber::Number(number) => number,
                    BitsOrNumber::Bits(..) => {
                        return Err(Box::new(
                            self.make_error("An imaginary component must be a number, not bits"),
                        ));
                    }
                }
            }
            _ => BigRational::zero(),
        };
        Ok(BitsOrNumber::Number(Complex::new(real, imaginary)))
    }

    fn numeric(&mut self, starts_with: char) -> Token {
        let token = match self.complex_or_bits(starts_with) {
            Ok(BitsOrNumber::Number(number)) => self.make_token(Numeric).with_value(number),
            Ok(BitsOrNumber::Bits(bits)) => self.make_token(Bits).with_value(bits),
            Err(error) => return *error,
        };
        if self.peek().is_some_and(|ch| ch.is_ascii_alphanumeric()) {
            return self.make_error("numeric literal may not have trailing characters");
        }
        token
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
        // Skip carriage returns entirely, they don't count as "whitespace"
        while self.expect("\r").is_some() {
            self.is_first_character = false;
        }
        // Emit one space token for each contiguous block of spaces and tabs
        if self.expect(" \t").is_some() {
            while self.expect(" \t").is_some() {}
            return Some(self.make_token(Space));
        }
        self.span.clear();
        if self.peek().is_none() {
            self.is_finished = true;
            return Some(self.make_token(EndOfFile));
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
            '\u{FEFF}' if self.is_first_character => self.make_token(ByteOrderMark),
            ch @ ('_' | 'a'..='z' | 'A'..='Z') => self.identifier_or_keyword(ch),
            '\'' => self.char_or_atom(),
            '#' => self.comment(),
            '"' => self.string_or_template(TemplateStart, String),
            // Feels slightly irresponsible to put side effects into a guard...
            // but it's been done all over this file. Apologies to reader.
            '$' => self.make_token(OpDollar),
            '!' if self.expect('=').is_some() => {
                if self.expect('=').is_some() {
                    self.make_token(OpBangEqEq)
                } else {
                    self.make_token(OpBangEq)
                }
            }
            '!' => self.make_token(OpBang),
            '{' if self.expect('|').is_some() => {
                self.nesting.push('/'); // using this arbitrarily as a sentinel for these, since | is taken
                self.make_token(OBracePipe)
            }
            '{' => {
                self.nesting.push('{');
                self.make_token(OBrace)
            }
            '}' if self.context('$') => self.string_or_template(TemplateContinue, TemplateEnd),
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
            '[' if self.expect('|').is_some() => {
                self.nesting.push('|');
                self.make_token(OBrackPipe)
            }
            '[' => {
                self.nesting.push('[');
                self.make_token(OBrack)
            }
            ']' => {
                if self.context('[') {
                    self.nesting.pop();
                }
                self.make_token(CBrack)
            }
            '\n' => self.make_token(EndOfLine),
            ch @ '0'..='9' => self.numeric(ch),

            '*' if self.expect('*').is_some() => {
                if self.expect('=').is_some() {
                    self.make_token(OpStarStarEq)
                } else {
                    self.make_token(OpStarStar)
                }
            }
            '*' if self.expect('=').is_some() => self.make_token(OpStarEq),
            '*' => self.make_token(OpStar),

            '+' if self.expect('=').is_some() => self.make_token(OpPlusEq),
            '+' => self.make_token(OpPlus),

            ',' => self.make_token(OpComma),

            '-' if self.expect('=').is_some() => self.make_token(OpMinusEq),
            '-' if self.expect('>').is_some() => self.make_token(OpRightArrow),
            '-' => self.make_token(OpMinus),

            '.' if self.expect('=').is_some() => self.make_token(OpDotEq),
            '.' if self.expect('.').is_some() => self.make_token(OpDotDot),
            '.' => self.make_token(OpDot),

            '?' => self.make_token(OpQuestion),

            '/' if self.expect('/').is_some() => {
                if self.expect('=').is_some() {
                    self.make_token(OpSlashSlashEq)
                } else {
                    self.make_token(OpSlashSlash)
                }
            }
            '/' if self.expect('=').is_some() => self.make_token(OpSlashEq),
            '/' => self.make_token(OpSlash),

            ':' if self.expect('=').is_some() => self.make_token(OpColonEq),
            ':' if self.expect(':').is_some() => self.make_token(OpColonColon),
            ':' => self.make_token(OpColon),
            ';' => self.make_token(OpSemi),

            '<' if self.expect('-').is_some() => self.make_token(OpLeftArrow),
            '<' if self.expect('<').is_some() => {
                if self.expect('~').is_some() {
                    if self.expect('=').is_some() {
                        self.make_token(OpShlConEq)
                    } else {
                        self.make_token(OpShlCon)
                    }
                } else if self.expect('=').is_some() {
                    self.make_token(OpLtLtEq)
                } else {
                    self.make_token(OpLtLt)
                }
            }
            '<' if self.expect('=').is_some() => self.make_token(OpLtEq),
            '<' if self.expect('>').is_some() => {
                if self.expect('=').is_some() {
                    self.make_token(OpGlueEq)
                } else {
                    self.make_token(OpGlue)
                }
            }
            '<' if self.expect('|').is_some() => self.make_token(OpLtPipe),
            '<' if self.expect('~').is_some() => {
                if self.expect('~').is_some() {
                    if self.expect('=').is_some() {
                        self.make_token(OpShlExEq)
                    } else {
                        self.make_token(OpShlEx)
                    }
                } else if self.expect('=').is_some() {
                    self.make_token(OpShlEq)
                } else {
                    self.make_token(OpShl)
                }
            }
            '<' => self.make_token(OpLt),

            '=' if self.expect('=').is_some() => {
                if self.expect('=').is_some() {
                    self.make_token(OpEqEqEq)
                } else {
                    self.make_token(OpEqEq)
                }
            }
            '=' if self.expect('>').is_some() => self.make_token(OpFatArrow),
            '=' => self.make_token(OpEq),

            '>' if self.expect('>').is_some() => {
                if self.expect('=').is_some() {
                    self.make_token(OpGtGtEq)
                } else {
                    self.make_token(OpGtGt)
                }
            }
            '>' if self.expect('=').is_some() => self.make_token(OpGtEq),
            '>' => self.make_token(OpGt),

            '%' if self.expect('=').is_some() => self.make_token(OpPercentEq),
            '%' => self.make_token(OpPercent),

            '&' if self.expect('&').is_some() => {
                if self.expect('=').is_some() {
                    self.make_token(OpAmpAmpEq)
                } else {
                    self.make_token(OpAmpAmp)
                }
            }
            '&' if self.expect('=').is_some() => self.make_token(OpAmpEq),
            '&' => self.make_token(OpAmp),

            '^' if self.expect('=').is_some() => self.make_token(OpCaretEq),
            '^' => self.make_token(OpCaret),

            '|' if self.expect(']').is_some() => {
                if self.context('|') {
                    self.nesting.pop();
                }
                self.make_token(CBrackPipe)
            }
            '|' if self.expect('}').is_some() => {
                if self.context('/') {
                    self.nesting.pop();
                }
                self.make_token(CBracePipe)
            }
            '|' if self.expect('|').is_some() => {
                if self.expect('=').is_some() {
                    self.make_token(OpPipePipeEq)
                } else {
                    self.make_token(OpPipePipe)
                }
            }
            '|' if self.expect('=').is_some() => self.make_token(OpPipeEq),
            '|' if self.expect('>').is_some() => self.make_token(OpPipeGt),
            '|' => self.make_token(OpPipe),

            '~' if self.expect('=').is_some() => self.make_token(OpTildeEq),
            '~' if self.peek() == Some('~') && self.predict('>') => {
                self.consume();
                self.consume();
                if self.expect('=').is_some() {
                    self.make_token(OpShrExEq)
                } else {
                    self.make_token(OpShrEx)
                }
            }
            '~' if self.expect('>').is_some() => {
                if self.expect('>').is_some() {
                    if self.expect('=').is_some() {
                        self.make_token(OpShrConEq)
                    } else {
                        self.make_token(OpShrCon)
                    }
                } else if self.expect('=').is_some() {
                    self.make_token(OpShrEq)
                } else {
                    self.make_token(OpShr)
                }
            }
            '~' => self.make_token(OpTilde),
            _ => self.make_error("Unexpected character found in input"),
        };
        self.is_first_character = false;
        if token.token_type == TemplateEnd && self.context('$') {
            self.nesting.pop();
        }
        if token.token_type == TemplateStart {
            self.nesting.push('$');
        }

        self.span.clear();
        Some(token)
    }
}

fn is_identifier(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
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
