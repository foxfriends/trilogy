use pretty::{DocBuilder, RcAllocator};

pub type PrettyPrinter = RcAllocator;
pub type PrettyPrinted<'a> = DocBuilder<'a, RcAllocator, ()>;

pub trait PrettyPrintSExpr<'a> {
    fn pretty_print_sexpr(&self, printer: &'a PrettyPrinter) -> PrettyPrinted<'a>;
}

pub trait PrettyPrint<'a> {
    fn pretty_print(&self, printer: &'a PrettyPrinter) -> PrettyPrinted<'a>;
}
