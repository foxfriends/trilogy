use super::super::Offset;
use super::error::{AsmError, ErrorKind, ValueError};
use super::from_asm_param::FromAsmParam;
use super::Asm;
use crate::runtime::atom::AtomInterner;
use crate::Value;
use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct AsmContext {
    line: usize,
    ip: usize,
    interner: AtomInterner,
    labels: HashMap<String, Offset>,
    holes: HashMap<usize, (Offset, String)>,
}

impl AsmContext {
    pub fn parse_offset(&mut self, src: &str) -> Result<usize, ErrorKind> {
        let offset = if let Some(suffix) = src.strip_prefix('&') {
            let label = Self::take_label(src);
            if label.is_empty() {
                return Err(ErrorKind::InvalidLabelReference);
            }
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

    fn take_label(src: &str) -> String {
        src.chars()
            .take_while(|&ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '@' || ch == '-')
            .collect()
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
        let prefix = Self::take_label(src);
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

    pub fn labels(self) -> HashMap<String, usize> {
        self.labels
    }

    pub fn holes(&self) -> impl Iterator<Item = Result<(usize, usize), AsmError>> + '_ {
        self.holes.iter().map(|(line, (hole, label))| {
            let offset = self.labels.get(label).ok_or_else(|| AsmError {
                line: *line,
                error: ErrorKind::MissingLabel(label.to_owned()),
            })?;
            Ok((*hole, usize::abs_diff(hole + 4, *offset)))
        })
    }

    fn is_empty(s: &str) -> bool {
        let s = s.trim_start();
        s.is_empty() || s.starts_with('#')
    }
}
