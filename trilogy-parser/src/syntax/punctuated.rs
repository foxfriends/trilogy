use crate::PrettyPrintSExpr;
use pretty::DocAllocator;
use trilogy_scanner::Token;

/// A punctuated list of nodes, separated by a punctuation token.
/// The list may have an optional trailing punctuator.
#[derive(Clone, Debug)]
pub struct Punctuated<T> {
    pub elements: Vec<(T, Token)>,
    pub last: Option<T>,
}

impl<'a, T: PrettyPrintSExpr<'a>> PrettyPrintSExpr<'a> for Punctuated<T> {
    fn pretty_print_sexpr(&self, printer: &'a crate::PrettyPrinter) -> crate::PrettyPrinted<'a> {
        printer
            .intersperse(
                self.iter().map(|node| node.pretty_print_sexpr(printer)),
                printer.line(),
            )
            .nest(2)
            .group()
            .brackets()
    }
}

impl<T> Default for Punctuated<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Punctuated<T> {
    pub(crate) const fn new() -> Self {
        Self {
            elements: vec![],
            last: None,
        }
    }

    pub(crate) fn init(item: T) -> Self {
        Self {
            elements: vec![],
            last: Some(item),
        }
    }

    pub(crate) fn follow(&mut self, punctuation: Token, item: T) {
        self.elements
            .push((self.last.replace(item).unwrap(), punctuation));
    }

    pub(crate) fn push(&mut self, item: T, punctuation: Token) {
        self.elements.push((item, punctuation));
    }

    pub(crate) fn push_last(&mut self, last: T) {
        self.last = Some(last);
    }

    pub(crate) fn finish(&mut self, last: Token) {
        self.elements.push((self.last.take().unwrap(), last));
    }

    /// An iterator over the useful elements in this punctuated list.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.into_iter()
    }
}

pub struct PunctuatedIterator<T> {
    elements: <Vec<(T, Token)> as IntoIterator>::IntoIter,
    last: Option<T>,
}

impl<T> IntoIterator for Punctuated<T> {
    type Item = T;
    type IntoIter = PunctuatedIterator<T>;

    fn into_iter(self) -> PunctuatedIterator<T> {
        PunctuatedIterator {
            elements: self.elements.into_iter(),
            last: self.last,
        }
    }
}

impl<T> Iterator for PunctuatedIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.elements
            .next()
            .map(|(t, _)| t)
            .or_else(|| self.last.take())
    }
}

pub struct PunctuatedIter<'a, T> {
    elements: <&'a Vec<(T, Token)> as IntoIterator>::IntoIter,
    last: Option<&'a T>,
}

impl<'a, T> IntoIterator for &'a Punctuated<T> {
    type Item = &'a T;
    type IntoIter = PunctuatedIter<'a, T>;

    fn into_iter(self) -> PunctuatedIter<'a, T> {
        PunctuatedIter {
            elements: self.elements.iter(),
            last: self.last.as_ref(),
        }
    }
}

impl<'a, T> Iterator for PunctuatedIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.elements
            .next()
            .map(|(t, _)| t)
            .or_else(|| self.last.take())
    }
}
