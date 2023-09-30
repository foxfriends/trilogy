use crate::bytecode::{Offset, OpCode};
use crate::runtime::atom::AtomInterner;
use string::extract_string_prefix;

mod error;
mod string;
mod value;

pub use error::AsmError;
use error::ErrorKind;

pub(crate) struct AsmReader<'a> {
    source: &'a str,
    position: usize,
    interner: AtomInterner,
}

pub(crate) enum Parameter {
    Label(String),
    Offset(Offset),
}

pub(crate) enum Value {
    Label(String),
    Value(crate::Value),
}

impl<'a> AsmReader<'a> {
    pub(crate) fn new(source: &'a str, interner: AtomInterner) -> Self {
        Self {
            source,
            position: 0,
            interner,
        }
    }

    fn error<K>(&self, kind: K) -> AsmError
    where
        ErrorKind: From<K>,
    {
        AsmError {
            position: self.position,
            kind: kind.into(),
        }
    }

    fn label(&mut self) -> Result<Option<String>, AsmError> {
        let src = &self.source[self.position..];
        if src.starts_with('"') {
            let (prefix, rest) = extract_string_prefix(src)
                .ok_or(ErrorKind::String)
                .map_err(|e| self.error(e))?;
            self.position += src.len() - rest.len();
            Ok(Some(prefix))
        } else {
            Ok(self.token())
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

    fn offset(&mut self) -> Result<Offset, AsmError> {
        let src = &self.source[self.position..];
        let (index, _) = src
            .char_indices()
            .find(|(_, ch)| !ch.is_numeric())
            .unwrap_or((src.len(), '\0'));
        let offset = src[..index].parse().map_err(|e| self.error(e))?;
        self.position += index;
        Ok(offset)
    }

    fn label_reference(&mut self) -> Result<String, AsmError> {
        let src = &self.source[self.position..];
        assert!(src.starts_with('&'));
        self.position += 1;
        match self.label() {
            Err(err) => {
                self.position -= 1;
                Err(err)
            }
            Ok(None) => {
                self.position -= 1;
                Err(self.error(ErrorKind::Label))
            }
            Ok(Some(label)) => Ok(label),
        }
    }

    pub fn label_definition(&mut self) -> Result<Option<String>, AsmError> {
        self.chomp();
        let start = self.position;
        let Some(label) = self.label()? else {
            return Ok(None);
        };
        if self.source[self.position..].starts_with(':') {
            self.position += 1;
            Ok(Some(label))
        } else {
            self.position = start;
            Ok(None)
        }
    }

    fn value_inner(&mut self) -> Result<crate::Value, AsmError> {
        let src = &self.source[self.position..];
        match crate::Value::parse_prefix(src, &self.interner) {
            Some((value, s)) => {
                self.position += src.len() - s.len();
                Ok(value)
            }
            None => Err(self.error(ErrorKind::Value)),
        }
    }

    pub fn value(&mut self) -> Result<Value, AsmError> {
        self.chomp();
        if self.source[self.position..].starts_with('&') {
            Ok(Value::Label(self.label_reference()?))
        } else {
            self.value_inner().map(Value::Value)
        }
    }

    pub fn parameter(&mut self) -> Result<Parameter, AsmError> {
        self.chomp();
        if self.source[self.position..].starts_with('&') {
            Ok(Parameter::Label(self.label_reference()?))
        } else {
            self.offset().map(Parameter::Offset)
        }
    }

    pub fn opcode(&mut self) -> Result<OpCode, AsmError> {
        self.token()
            .ok_or_else(|| self.error(ErrorKind::Token))?
            .parse()
            .map_err(|e| self.error(e))
    }

    pub fn is_empty(&mut self) -> bool {
        self.chomp();
        self.position == self.source.len()
    }
}
