use super::*;
use crate::Analyzer;
use trilogy_lexical_ir::Module;
use trilogy_parser::syntax::ModuleDefinition;
use trilogy_parser::Spanned;

pub(super) fn analyze_module(analyzer: &mut Analyzer, ast: ModuleDefinition) -> Module {
    analyze_definitions(analyzer, ast.span(), ast.definitions)
}
