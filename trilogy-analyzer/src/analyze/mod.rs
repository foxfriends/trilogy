mod definitions;
mod document;
mod func;
mod module;
mod module_path;
mod proc;
mod prose;
mod rule;
mod test;

use definitions::analyze_definitions;
pub(crate) use document::analyze_document as analyze;
use func::analyze_func;
use module::analyze_module;
use module_path::analyze_module_path;
use proc::analyze_proc;
use prose::analyze_prose;
use rule::analyze_rule;
use test::analyze_test;