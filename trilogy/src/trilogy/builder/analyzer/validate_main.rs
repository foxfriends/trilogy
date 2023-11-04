use super::prelude::*;
use trilogy_ir::ir;

pub(super) fn validate_main<E: std::error::Error>(
    modules: &mut Modules,
    entrypoint: &Location,
    report: &mut ReportBuilder<E>,
) {
    let entrymodule = modules.get_mut(entrypoint);
    let main = entrymodule
        .unwrap()
        .definitions_mut()
        .iter_mut()
        .find(|def| {
            def.name()
                .and_then(|id| id.name())
                .map(|name| name == "main")
                .unwrap_or(false)
        });
    match main {
        None => report.error(Error::analysis(
            entrypoint.clone(),
            ErrorKind::NoMainProcedure,
        )),
        Some(def) => match &def.item {
            ir::DefinitionItem::Procedure(..) => {
                // Force main to be exported. It needs to be accessible because
                // programs are really just modules with a wrapper that automatically
                // imports and calls `main`.
                def.is_exported = true;
            }
            item => report.error(Error::analysis(
                entrypoint.clone(),
                ErrorKind::MainNotProcedure { item: item.clone() },
            )),
        },
    }
}
