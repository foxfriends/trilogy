use super::report::ReportBuilder;
use crate::Location;
use std::collections::HashMap;
use trilogy_ir::ir::Module;

mod error_kind;

mod validate_constants;
mod validate_main;

pub(crate) use error_kind::ErrorKind;

pub(super) fn analyze<E: std::error::Error>(
    modules: &mut HashMap<Location, Module>,
    entrypoint: &Location,
    report: &mut ReportBuilder<E>,
    is_library: bool,
) {
    if !is_library {
        validate_main::validate_main(modules, entrypoint, report);
    }
    // validate_constants::validate_constants(modules, report);
}

mod prelude {
    pub(super) use super::super::error::Error;
    pub(super) use super::super::report::ReportBuilder;
    pub(super) use super::error_kind::ErrorKind;
    pub(super) use crate::Location;

    use std::collections::HashMap;
    use trilogy_ir::ir::Module;
    pub(super) type Modules = HashMap<Location, Module>;
}
