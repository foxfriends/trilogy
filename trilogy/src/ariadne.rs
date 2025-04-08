use crate::location::Location;
use ariadne::FnCache;
use source_span::Span;
use std::fmt::{self, Debug};
use std::ops::Range;
use std::path::Path;

pub(crate) trait CacheExt<I>: ariadne::Cache<I> {
    fn span(&mut self, location: &I, span: Span) -> (I, Range<usize>)
    where
        I: Clone,
    {
        match self.fetch(location) {
            Ok(source) => {
                let Some(start_line) = source.line(span.start().line) else {
                    return (location.clone(), 0..0);
                };
                let Some(end_line) = source.line(span.end().line) else {
                    return (location.clone(), 0..0);
                };
                let start = start_line.offset() + span.start().column;
                let end = end_line.offset() + span.end().column;
                (location.clone(), start..end)
            }
            Err(..) => (location.clone(), 0..0),
        }
    }
}

impl<I, C> CacheExt<I> for C where C: ariadne::Cache<I> {}

pub(crate) struct LoaderCache<'a, F, I: AsRef<str>> {
    relative_base: &'a Path,
    inner: FnCache<Location, F, I>,
}

impl<'a, F, I: AsRef<str>> LoaderCache<'a, F, I> {
    pub(crate) fn new(relative_base: &'a Path, inner: FnCache<Location, F, I>) -> Self {
        Self {
            relative_base,
            inner,
        }
    }
}

impl<F, I> ariadne::Cache<Location> for LoaderCache<'_, F, I>
where
    F: for<'b> FnMut(&'b Location) -> Result<I, Box<dyn Debug>>,
    I: AsRef<str>,
{
    type Storage = I;

    fn fetch(&mut self, id: &Location) -> Result<&ariadne::Source<Self::Storage>, impl fmt::Debug> {
        self.inner.fetch(id)
    }

    fn display<'a>(&self, id: &'a Location) -> Option<impl fmt::Display + 'a> {
        if id.as_ref().scheme() != "file" {
            return Some(Box::new(id.to_owned().to_string()));
        }
        match Path::new(id.as_ref().path()).strip_prefix(self.relative_base) {
            Ok(path) => Some(Box::new(path.display().to_string())),
            Err(..) => Some(Box::new(id.to_owned().to_string())),
        }
    }
}
