use pretty::{DocAllocator, DocBuilder, RcAllocator};
use trilogy_scanner::Token;

pub type PrettyPrinter = RcAllocator;
pub type PrettyPrinted<'a> = DocBuilder<'a, RcAllocator, ()>;

pub trait PrettyPrintSExpr<'a> {
    fn pretty_print_sexpr(&self, printer: &'a PrettyPrinter) -> PrettyPrinted<'a>;
}

impl<'a, P> PrettyPrintSExpr<'a> for Option<P>
where
    P: PrettyPrintSExpr<'a>,
{
    fn pretty_print_sexpr(&self, printer: &'a PrettyPrinter) -> PrettyPrinted<'a> {
        match self {
            Some(node) => node.pretty_print_sexpr(printer),
            None => printer.nil().parens(),
        }
    }
}

impl<'a, P> PrettyPrintSExpr<'a> for Vec<P>
where
    P: PrettyPrintSExpr<'a>,
{
    fn pretty_print_sexpr(&self, printer: &'a PrettyPrinter) -> PrettyPrinted<'a> {
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

impl<'a, P, Q> PrettyPrintSExpr<'a> for (P, Q)
where
    P: PrettyPrintSExpr<'a>,
    Q: PrettyPrintSExpr<'a>,
{
    fn pretty_print_sexpr(&self, printer: &'a PrettyPrinter) -> PrettyPrinted<'a> {
        printer
            .nil()
            .append(self.0.pretty_print_sexpr(printer))
            .append(printer.line())
            .append(self.1.pretty_print_sexpr(printer))
    }
}

impl<'a> PrettyPrintSExpr<'a> for Token {
    fn pretty_print_sexpr(&self, printer: &'a PrettyPrinter) -> PrettyPrinted<'a> {
        printer.text(format!("{:?}", self.token_type))
    }
}
