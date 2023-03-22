mod assert_statement;
mod definitions;
mod document;
mod func;
mod function_assignment;
mod lvalue;
mod module;
mod module_path;
mod poetry;
mod proc;
mod prose;
mod rule;
mod statement;
mod test;

use assert_statement::analyze_assert_statement;
use definitions::analyze_definitions;
pub(crate) use document::analyze_document as analyze;
use func::analyze_func;
use function_assignment::analyze_function_assignment;
use lvalue::analyze_lvalue;
use module::analyze_module;
use module_path::analyze_module_path;
use poetry::analyze_poetry;
use proc::analyze_proc;
use prose::analyze_prose;
use rule::analyze_rule;
use statement::analyze_statement;
use test::analyze_test;
