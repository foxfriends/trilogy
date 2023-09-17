mod string;
mod value;

use crate::runtime::atom::AtomInterner;
use crate::{Offset, OpCode, Value};
use string::extract_string_prefix;

pub(crate) struct AsmReader<'a> {
    source: &'a str,
    position: usize,
    interner: AtomInterner,
}

pub(crate) enum Parameter {
    Label(String),
    Offset(Offset),
}

impl<'a> AsmReader<'a> {
    pub(crate) fn new(source: &'a str, interner: AtomInterner) -> Self {
        Self {
            source,
            position: 0,
            interner,
        }
    }

    fn label(&mut self) -> Option<String> {
        let src = &self.source[self.position..];
        if src.starts_with('"') {
            let (prefix, rest) = extract_string_prefix(src)?;
            self.position += src.len() - rest.len();
            Some(prefix)
        } else {
            self.token()
        }
    }

    fn token(&mut self) -> Option<String> {
        let src = &self.source[self.position..];
        let (index, _) = src
            .char_indices()
            .find(|&(_, ch)| !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '@' || ch == '-'))
            .unwrap_or((src.len(), '\0'));
        if index == 0 {
            return None;
        }
        self.position += index;
        Some(src[..index].to_owned())
    }

    fn chomp(&mut self) {
        loop {
            match self.source[self.position..].chars().next() {
                Some(ch) if ch.is_ascii_whitespace() => {
                    self.position += 1;
                }
                Some('#') => {
                    match self.source[self.position..]
                        .char_indices()
                        .find(|(_, ch)| *ch == '\n')
                    {
                        Some((index, _)) => self.position += index + 1,
                        None => self.position = self.source.len(),
                    }
                }
                _ => return,
            }
        }
    }

    fn offset(&mut self) -> Option<Offset> {
        let src = &self.source[self.position..];
        let (index, _) = src
            .char_indices()
            .find(|(_, ch)| !ch.is_numeric())
            .unwrap_or((src.len(), '\0'));
        let offset = src[..index].parse().ok()?;
        self.position += index;
        Some(offset)
    }

    fn label_reference(&mut self) -> Option<String> {
        let src = &self.source[self.position..];
        if !src.starts_with('&') {
            return None;
        }
        self.position += 1;
        self.label().or_else(|| {
            self.position -= 1;
            None
        })
    }

    pub fn label_definition(&mut self) -> Option<String> {
        self.chomp();
        let start = self.position;
        let label = self.label()?;
        if self.source[self.position..].starts_with(':') {
            self.position += 1;
            Some(label)
        } else {
            self.position = start;
            None
        }
    }

    pub fn value(&mut self) -> Option<Value> {
        self.chomp();
        let src = &self.source[self.position..];
        match Value::parse_prefix(src, &self.interner) {
            Some((value, s)) => {
                self.position += src.len() - s.len();
                Some(value)
            }
            None => None,
        }
    }

    pub fn parameter(&mut self) -> Option<Parameter> {
        self.chomp();
        if self.source[self.position..].starts_with('&') {
            self.label_reference().map(Parameter::Label)
        } else {
            self.offset().map(Parameter::Offset)
        }
    }

    pub fn opcode(&mut self) -> Option<OpCode> {
        self.token()?.parse().ok()
    }

    pub fn is_empty(&mut self) -> bool {
        self.chomp();
        self.position == self.source.len()
    }
}
