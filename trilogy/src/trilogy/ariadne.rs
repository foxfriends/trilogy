use super::error::ErrorKind;
use super::loader::Loader;
use super::Error;
use crate::location::Location;
use crate::Cache;
use ariadne::{ColorGenerator, Config, Fmt, FnCache, Label, ReportKind};
use source_span::Span;
use std::collections::HashMap;
use std::fmt::{self, Debug};
use std::ops::Range;
use std::path::{Path, PathBuf};
use trilogy_ir::ir::DefinitionItem;
use trilogy_parser::Spanned;

trait CacheExt<'a>: ariadne::Cache<&'a Location> {
    fn span(&mut self, location: &'a Location, span: Span) -> (&'a Location, Range<usize>) {
        match self.fetch(&location) {
            Ok(source) => {
                let start = source.line(span.start().line).unwrap().offset() + span.start().column;
                let end = source.line(span.end().line).unwrap().offset() + span.end().column;
                (location, start..end)
            }
            Err(..) => (location, 0..0),
        }
    }
}

impl<'a, C> CacheExt<'a> for C where C: ariadne::Cache<&'a Location> {}

pub(crate) struct LoaderCache<'a, F> {
    relative_base: &'a Path,
    inner: FnCache<&'a Location, F>,
}

impl<'a, F> ariadne::Cache<&'a Location> for LoaderCache<'a, F>
where
    F: for<'b> FnMut(&'b &'a Location) -> Result<String, Box<dyn Debug>>,
{
    fn fetch(&mut self, id: &&'a Location) -> Result<&ariadne::Source, Box<dyn fmt::Debug + '_>> {
        self.inner.fetch(id)
    }

    fn display<'b>(&self, id: &'b &'a Location) -> Option<Box<dyn fmt::Display + 'a>> {
        if id.as_ref().scheme() != "file" {
            return Some(Box::new(id.to_owned()));
        }
        match Path::new(id.as_ref().path()).strip_prefix(self.relative_base) {
            Ok(path) => Some(Box::new(path.display().to_string())),
            Err(..) => Some(Box::new(id.to_owned())),
        }
    }
}
