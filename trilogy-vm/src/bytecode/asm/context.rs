use super::super::Offset;
use super::error::{AsmError, ErrorKind, ValueError};
use super::from_asm_param::FromAsmParam;
use super::string::extract_string_prefix;
use super::Asm;
use crate::runtime::atom::AtomInterner;
use crate::{Atom, Value};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct AsmContext {
    line: usize,
    ip: usize,
    interner: AtomInterner,
    labels: HashMap<String, Offset>,
    holes: HashMap<usize, (Offset, String)>,
    value_holes: HashMap<usize, (Offset, String)>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct LabelAlreadyInserted;

impl AsmContext {
    pub fn insert_label(&mut self, label: String) -> Result<Offset, LabelAlreadyInserted> {
        if let Entry::Vacant(e) = self.labels.entry(label) {
            e.insert(self.ip);
            Ok(self.ip)
        } else {
            Err(LabelAlreadyInserted)
        }
    }

    pub fn take_label(src: &str) -> Option<(String, &str)> {
        if src.starts_with('"') {
            // A bit funny but we keep the quotes on these... Just makes life easier since
            // the thing consuming this label is not receiving tail slice right now. Nobody
            // really sees the labels anyway, unless things crash.
            extract_string_prefix(src)
        } else {
            let label: String = src
                .chars()
                .take_while(|&ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '@' || ch == '-')
                .collect();
            if label.is_empty() {
                None
            } else {
                let src = &src[label.len()..];
                Some((label, src))
            }
        }
    }

    pub(super) fn parse_offset(&mut self, src: &str) -> Result<usize, ErrorKind> {
        let offset = if let Some(suffix) = src.strip_prefix('&') {
            let (label, _) = Self::take_label(suffix).ok_or(ErrorKind::InvalidLabelReference)?;
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

    pub(super) fn lookup_label(&mut self, s: &str) -> Option<Offset> {
        self.labels.get(s).copied()
    }

    pub fn intern(&mut self, atom: &str) -> Atom {
        self.interner.intern(atom)
    }

    pub(super) fn parse_value(&mut self, src: &str) -> Result<Value, ValueError> {
        let value = match self.parse_value_final(src) {
            Err(ValueError::UnresolvedLabelReference) => {
                self.value_holes
                    .insert(self.line, (self.ip, src.to_owned()));
                Value::Unit
            }
            value => value?,
        };
        self.ip += 4;
        Ok(value)
    }

    fn parse_value_final(&mut self, src: &str) -> Result<Value, ValueError> {
        let value = match Value::parse_prefix(src, self) {
            Ok((value, s)) if Self::is_empty(s) => value,
            Ok(..) => return Err(ValueError::ExtraChars),
            Err(error) => return Err(error),
        };
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
        if let Some((prefix, src)) = Self::take_label(src) {
            if let Some(src) = src.strip_prefix(':') {
                self.labels.insert(prefix, self.ip);
                return self.parse_line(src);
            }
        }
        self.ip += 1;
        T::parse_asm(src, self)
            .map_err(|error| AsmError {
                line: self.line,
                error,
            })
            .map(Some)
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

    pub fn labels(self) -> HashMap<String, usize> {
        self.labels
    }

    pub fn holes(&self) -> impl Iterator<Item = Result<(Offset, Offset), AsmError>> + '_ {
        self.holes.iter().map(|(line, (hole, label))| {
            let offset = self.labels.get(label).ok_or_else(|| AsmError {
                line: *line,
                error: ErrorKind::MissingLabel(label.to_owned()),
            })?;
            // Jumps are taken from the beginning of the next instruction, which is 4 bytes
            // after the start of the hole.
            Ok((*hole, usize::abs_diff(hole + 4, *offset)))
        })
    }

    pub fn value_holes(&mut self) -> impl Iterator<Item = Result<(Offset, Value), AsmError>> + '_ {
        let value_holes = std::mem::take(&mut self.value_holes);
        value_holes.into_iter().map(|(line, (hole, src))| {
            let value = self.parse_value_final(&src).map_err(|error| AsmError {
                line,
                error: ErrorKind::InvalidValue(error),
            })?;
            Ok((hole, value))
        })
    }

    fn is_empty(s: &str) -> bool {
        let s = s.trim_start();
        s.is_empty() || s.starts_with('#')
    }
}
