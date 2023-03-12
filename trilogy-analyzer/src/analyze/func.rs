use crate::Analyzer;
use trilogy_lexical_ir::Item;
use trilogy_parser::syntax::FunctionDefinition;
use trilogy_parser::Spanned;

#[allow(unreachable_code)]
pub(super) fn analyze_func(_analyzer: &mut Analyzer, func: FunctionDefinition) -> Item {
    Item {
        span: func.span(),
        source: todo!(),
    }
}
