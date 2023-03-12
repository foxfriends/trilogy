use super::*;
use crate::Analyzer;
use trilogy_lexical_ir::{BinaryOperation, Evaluation, Value};
use trilogy_parser::syntax::{Identifier, ModulePath};
use trilogy_parser::Spanned;

fn static_resolve(ident: Identifier) -> Evaluation {
    Evaluation {
        span: ident.span(),
        value: Value::StaticResolve(ident.into()),
    }
}

pub(super) fn analyze_module_path(analyzer: &mut Analyzer, module_path: ModulePath) -> Evaluation {
    let mut modules = module_path.modules.into_iter();
    let first = modules
        .next()
        .expect("module path should contain at least one module reference");
    let module = static_resolve(first.name);
    modules.fold(module, |evaluation, module_reference| {
        let module_function = Evaluation {
            span: module_reference.span(),
            value: Value::AccessModule(Box::new(BinaryOperation::new(
                evaluation,
                static_resolve(module_reference.name),
            ))),
        };
        module_reference
            .arguments
            .into_iter()
            .fold(module_function, |func, argument| {
                let span = func.span.union(argument.span());
                let expression = analyze_poetry(analyzer, argument);
                Evaluation {
                    span,
                    value: Value::ApplyModule(Box::new(BinaryOperation::new(func, expression))),
                }
            })
    })
}
