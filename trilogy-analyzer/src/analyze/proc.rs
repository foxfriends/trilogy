use crate::Analyzer;
use trilogy_lexical_ir::Item;
use trilogy_parser::syntax::ProcedureDefinition;
use trilogy_parser::Spanned;

#[allow(unreachable_code)]
pub(super) fn analyze_proc(_analyzer: &mut Analyzer, proc: ProcedureDefinition) -> Item {
    Item {
        span: proc.span(),
        source: todo!(),
    }
}
