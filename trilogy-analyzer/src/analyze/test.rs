use super::*;
use crate::Analyzer;
use trilogy_lexical_ir::Test;
use trilogy_parser::syntax::TestDefinition;
use trilogy_parser::Spanned;

pub(super) fn analyze_test(analyzer: &mut Analyzer, test: TestDefinition) -> Test {
    let span = test.span();
    let name = test.name.into();
    let code = analyze_prose(analyzer, test.body);
    Test { span, name, code }
}
