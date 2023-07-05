use super::Offset;
use crate::{runtime::atom::AtomInterner, Array, Bits, Record, Set, Struct, Tuple, Value};
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(crate) struct AsmContext {
    line: usize,
    ip: usize,
    interner: AtomInterner,
    labels: HashMap<String, Offset>,
    holes: HashMap<usize, (Offset, String)>,
}

#[derive(Clone, Debug)]
pub struct AsmError {
    pub line: usize,
    pub error: ErrorKind,
}

#[derive(Clone, Debug)]
pub enum ErrorKind {
    UnknownOpcode(String),
    MissingParameter,
    InvalidOffset,
    InvalidLabelReference,
    MissingLabel(String),
    InvalidValue(ValueError),
}

pub(crate) trait FromAsmParam: Sized {
    fn from_asm_param(src: &str, ctx: &mut AsmContext) -> Result<Self, ErrorKind>;
}

impl AsmContext {
    pub fn parse_offset(&mut self, src: &str) -> Result<usize, ErrorKind> {
        let offset = if let Some(suffix) = src.strip_prefix('&') {
            let label: String = suffix
                .chars()
                .take_while(|&ch| ch.is_ascii_alphanumeric() || ch == '_')
                .collect();
            if Self::is_empty(&suffix[label.len()..]) {
                self.holes.insert(self.line, (self.ip, label));
                0
            } else {
                return Err(ErrorKind::InvalidLabelReference);
            }
        } else {
            src.parse().map_err(|_| ErrorKind::InvalidOffset)?
        };

        self.ip += 4;

        Ok(offset)
    }

    pub fn parse_value(&mut self, src: &str) -> Result<Value, ValueError> {
        let value = match Value::parse_prefix(src, &mut self.interner) {
            Ok((value, s)) if Self::is_empty(s) => value,
            Ok(..) => return Err(ValueError::ExtraChars),
            Err(error) => return Err(error),
        };
        self.ip += 4;
        Ok(value)
    }

    pub fn parse_param<T: FromAsmParam>(&mut self, src: Option<&str>) -> Result<T, ErrorKind> {
        let src = src.ok_or(ErrorKind::MissingParameter)?;
        T::from_asm_param(src, self)
    }

    pub fn parse_line<T>(&mut self, src: &str) -> Result<Option<T>, AsmError>
    where
        T: Asm,
    {
        if Self::is_empty(src) {
            return Ok(None);
        }
        let src = src.trim_start();
        let prefix: String = src
            .chars()
            .take_while(|&ch| ch.is_ascii_alphanumeric() || ch == '_')
            .collect();
        if let Some(src) = src[prefix.len()..].strip_prefix(':') {
            self.labels.insert(prefix, self.ip);
            self.parse_line(src)
        } else {
            self.ip += 1;
            T::parse_asm(src, self)
                .map_err(|error| AsmError {
                    line: self.line,
                    error,
                })
                .map(Some)
        }
    }

    pub fn parse<'a, T>(
        &'a mut self,
        src: &'a str,
    ) -> impl Iterator<Item = Result<T, AsmError>> + 'a
    where
        T: Asm,
    {
        src.lines().enumerate().filter_map(|(line, src)| {
            self.line = line;
            self.parse_line(src).transpose()
        })
    }

    pub fn holes(self) -> impl Iterator<Item = Result<(usize, usize), AsmError>> {
        self.holes.into_iter().map(move |(line, (hole, label))| {
            let offset = self.labels.get(&label).ok_or(AsmError {
                line,
                error: ErrorKind::MissingLabel(label),
            })?;
            Ok((hole, usize::abs_diff(hole + 4, *offset)))
        })
    }

    fn is_empty(s: &str) -> bool {
        let s = s.trim_start();
        s.is_empty() || s.starts_with('#')
    }
}

impl FromAsmParam for Value {
    fn from_asm_param(src: &str, ctx: &mut AsmContext) -> Result<Self, ErrorKind> {
        Ok(ctx.parse_value(src)?)
    }
}

impl FromAsmParam for Offset {
    fn from_asm_param(src: &str, ctx: &mut AsmContext) -> Result<Self, ErrorKind> {
        ctx.parse_offset(src)
    }
}

pub(crate) trait Asm: Sized {
    fn fmt_asm(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result;
    fn parse_asm(src: &str, ctx: &mut AsmContext) -> Result<Self, ErrorKind>;
}

impl From<ValueError> for ErrorKind {
    fn from(value: ValueError) -> Self {
        Self::InvalidValue(value)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ValueError {
    InvalidProcedure,
    InvalidCharacter,
    InvalidAtom,
    InvalidTuple,
    InvalidNumber,
    InvalidStruct,
    InvalidArray,
    InvalidString,
    InvalidRecord,
    InvalidSet,
    ExtraChars,
}

impl Value {
    fn parse_prefix<'a>(
        s: &'a str,
        interner: &mut AtomInterner,
    ) -> Result<(Self, &'a str), ValueError> {
        match s {
            _ if s.starts_with("unit") => Ok((Value::Unit, &s[4..])),
            _ if s.starts_with("true") => Ok((Value::Bool(true), &s[4..])),
            _ if s.starts_with("false") => Ok((Value::Bool(false), &s[5..])),
            _ if s.starts_with('\'') => {
                if s.starts_with('\\') {
                    let (ch, s) = Self::escape_sequence(s).ok_or(ValueError::InvalidCharacter)?;
                    let s = s.strip_prefix('\'').ok_or(ValueError::InvalidCharacter)?;
                    Ok((Value::Char(ch), s))
                } else if &s[2..3] == "'" {
                    Ok((
                        Value::Char(s[1..2].parse().map_err(|_| ValueError::InvalidCharacter)?),
                        &s[3..],
                    ))
                } else {
                    let atom: String = s[1..]
                        .chars()
                        .take_while(|&ch| ch.is_ascii_alphanumeric() || ch == '_')
                        .collect();
                    if atom.is_empty() {
                        Err(ValueError::InvalidAtom)
                    } else {
                        let s = &s[atom.len()..];
                        let atom = interner.intern(&atom);
                        if let Some(s) = s.strip_prefix('(') {
                            let (value, s) = Value::parse_prefix(s, interner)?;
                            let s = s.strip_prefix(')').ok_or(ValueError::InvalidStruct)?;
                            Ok((Value::Struct(Struct::new(atom, value)), s))
                        } else {
                            Ok((Value::Atom(atom), s))
                        }
                    }
                }
            }
            _ if s.starts_with('(') => {
                let s = &s[1..];
                let (lhs, s) = Value::parse_prefix(s, interner)?;
                let s = s.strip_prefix(':').ok_or(ValueError::InvalidTuple)?;
                let (rhs, s) = Value::parse_prefix(s, interner)?;
                let s = s.strip_prefix(')').ok_or(ValueError::InvalidTuple)?;
                Ok((Value::Tuple(Tuple::new(lhs, rhs)), s))
            }
            _ if s.starts_with('"') => {
                let mut string = String::new();
                let mut s = &s[1..];
                loop {
                    if s.is_empty() {
                        return Err(ValueError::InvalidString);
                    }
                    if let Some(s) = s.strip_prefix('"') {
                        return Ok((Value::String(string), s));
                    }
                    if s.starts_with('\\') {
                        let (ch, rest) =
                            Self::escape_sequence(s).ok_or(ValueError::InvalidString)?;
                        s = rest;
                        string.push(ch);
                        continue;
                    }
                    string.push(s.chars().next().ok_or(ValueError::InvalidString)?);
                    s = &s[1..];
                }
            }
            _ if s.starts_with("[|") => {
                let mut set = HashSet::new();
                let mut s = &s[2..];
                let s = loop {
                    if let Some(rest) = s.strip_prefix("|]") {
                        break rest;
                    }
                    if s.is_empty() {
                        return Err(ValueError::InvalidSet);
                    }
                    let (value, rest) = Value::parse_prefix(s, interner)?;
                    set.insert(value);
                    if let Some(rest) = rest.strip_prefix("|]") {
                        break rest;
                    }
                    s = rest.strip_prefix(',').ok_or(ValueError::InvalidSet)?;
                };
                Ok((Value::Set(Set::from(set)), s))
            }
            _ if s.starts_with("{|") => {
                let mut map = HashMap::new();
                let mut s = &s[2..];
                let s = loop {
                    if let Some(rest) = s.strip_prefix("|}") {
                        break rest;
                    }
                    if s.is_empty() {
                        return Err(ValueError::InvalidRecord);
                    }
                    let (key, rest) = Value::parse_prefix(s, interner)?;
                    let rest = rest.strip_prefix("=>").ok_or(ValueError::InvalidRecord)?;
                    let (value, rest) = Value::parse_prefix(rest, interner)?;
                    map.insert(key, value);
                    if let Some(rest) = rest.strip_prefix("|}") {
                        break rest;
                    }
                    s = rest.strip_prefix(',').ok_or(ValueError::InvalidRecord)?;
                };
                Ok((Value::Record(Record::from(map)), s))
            }
            _ if s.starts_with('[') => {
                let mut array = vec![];
                let mut s = &s[1..];
                let s = loop {
                    if let Some(rest) = s.strip_prefix(']') {
                        break rest;
                    }
                    if s.is_empty() {
                        return Err(ValueError::InvalidArray);
                    }
                    let (value, rest) = Value::parse_prefix(s, interner)?;
                    array.push(value);
                    if let Some(rest) = rest.strip_prefix(']') {
                        break rest;
                    }
                    s = rest.strip_prefix(',').ok_or(ValueError::InvalidArray)?;
                };
                Ok((Value::Array(Array::from(array)), s))
            }
            _ if s.starts_with('&') => {
                let s = &s[1..];
                let numberlike: String = s.chars().take_while(|ch| ch.is_numeric()).collect();
                let offset = numberlike
                    .parse()
                    .map_err(|_| ValueError::InvalidProcedure)?;
                Ok((Value::Procedure(offset), &s[numberlike.len()..]))
            }
            _ if s.starts_with("0b") => {
                let bits: Bits = s[2..]
                    .chars()
                    .take_while(|&c| c == '0' || c == '1')
                    .map(|ch| ch == '1')
                    .collect();
                let s = &s[bits.len() + 2..];
                Ok((Value::Bits(bits), s))
            }
            s => {
                let numberlike: String = s
                    .chars()
                    .take_while(|&c| {
                        // All the valid characters of these complex big rationals
                        c.is_numeric()
                            || c == '-'
                            || c == '+'
                            || c == 'i'
                            || c == 'e'
                            || c == '.'
                            || c == 'E'
                            || c == '/'
                    })
                    .collect();
                Ok((
                    Value::Number(numberlike.parse().map_err(|_| ValueError::InvalidNumber)?),
                    &s[numberlike.len()..],
                ))
            }
        }
    }

    // NOTE: Logic taken from scanner

    fn unicode_escape_sequence(s: &str) -> Option<(char, &str)> {
        let s = s.strip_prefix('{')?;
        let repr: String = s.chars().take_while(|ch| ch.is_ascii_hexdigit()).collect();
        let s = s[repr.len()..].strip_prefix('}')?;
        let num = u32::from_str_radix(&repr, 16).ok()?;
        Some((char::from_u32(num)?, s))
    }

    fn ascii_escape_sequence(s: &str) -> Option<(char, &str)> {
        u32::from_str_radix(&s[0..2], 16)
            .ok()
            .and_then(char::from_u32)
            .map(|ch| (ch, &s[2..]))
    }

    fn escape_sequence(s: &str) -> Option<(char, &str)> {
        match s.strip_prefix('\\')? {
            s if s.starts_with('u') => Self::unicode_escape_sequence(&s[1..]),
            s if s.starts_with('x') => Self::ascii_escape_sequence(&s[1..]),
            s if s.starts_with(|ch| matches!(ch, '"' | '\'' | '$' | '\\')) => {
                Some((s.chars().next()?, &s[1..]))
            }
            s if s.starts_with('n') => Some(('\n', &s[1..])),
            s if s.starts_with('t') => Some(('\t', &s[1..])),
            s if s.starts_with('r') => Some(('\r', &s[1..])),
            s if s.starts_with('0') => Some(('\0', &s[1..])),
            _ => None,
        }
    }
}
