use super::error::Error;
use super::report::ReportBuilder;
use crate::Location;
use std::collections::HashMap;
use trilogy_ir::ir::Module;

mod error_kind;

pub(crate) use error_kind::ErrorKind;

pub(super) fn analyze<E: std::error::Error>(
    modules: &mut HashMap<Location, Module>,
    entrypoint: &Location,
    report: &mut ReportBuilder<E>,
) {
    // A bit hacky to be constructing IR errors here, and not in the IR crate,
    // but... whatever, it's easier, and this is a nice one-off check for now.
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
            trilogy_ir::ir::DefinitionItem::Procedure(..) => {
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
