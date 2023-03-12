use crate::Analyzer;
use trilogy_lexical_ir::Item;
use trilogy_parser::syntax::RuleDefinition;
use trilogy_parser::Spanned;

#[allow(unreachable_code)]
pub(super) fn analyze_rule(_analyzer: &mut Analyzer, rule: RuleDefinition) -> Item {
    Item {
        span: rule.span(),
        source: todo!(),
    }
}
