use inkwell::{
    debug_info::{
        AsDIScope, DIBasicType, DICompileUnit, DILexicalBlock, DIScope, DISubroutineType,
        DWARFEmissionKind, DWARFSourceLanguage, DebugInfoBuilder,
    },
    llvm_sys::debuginfo::LLVMDIFlagPublic,
    module::Module,
};
use source_span::Span;
use std::cell::RefCell;

use crate::codegen::Codegen;

pub(crate) struct DebugInfo<'ctx> {
    pub(crate) builder: DebugInfoBuilder<'ctx>,
    pub(crate) unit: DICompileUnit<'ctx>,
    pub(crate) debug_scopes: RefCell<Vec<DIScope<'ctx>>>,
}

impl<'ctx> DebugInfo<'ctx> {
    pub(crate) fn new(module: &Module<'ctx>, filename: &str, directory: &str) -> Self {
        let (builder, unit) = module.create_debug_info_builder(
            true,
            DWARFSourceLanguage::C,
            filename,
            directory,
            "trilogy",
            false,
            "",
            0,
            "",
            DWARFEmissionKind::Full,
            0,
            false,
            false,
            "",
            "",
        );
        DebugInfo {
            builder,
            unit,
            debug_scopes: RefCell::new(vec![unit.as_debug_info_scope()]),
        }
    }

    pub(crate) fn value_di_type(&self) -> DIBasicType<'ctx> {
        self.builder
            .create_basic_type("trilogy_value", 8 * 9, 0, LLVMDIFlagPublic)
            .unwrap()
    }

    pub(crate) fn procedure_di_type(&self, arity: usize) -> DISubroutineType<'ctx> {
        self.builder.create_subroutine_type(
            self.unit.get_file(),
            Some(self.value_di_type().as_type()),
            &vec![self.value_di_type().as_type(); arity],
            LLVMDIFlagPublic,
        )
    }

    pub(crate) fn push_debug_scope(&self, scope: DIScope<'ctx>) {
        self.debug_scopes.borrow_mut().push(scope);
    }

    pub(crate) fn push_block_scope(&self, span: Span) -> DILexicalBlock<'ctx> {
        let scope = self.get_debug_scope().unwrap();
        let block = self.builder.create_lexical_block(
            scope,
            self.unit.get_file(),
            span.start().line as u32,
            span.start().column as u32,
        );
        self.debug_scopes
            .borrow_mut()
            .push(block.as_debug_info_scope());
        block
    }

    pub(crate) fn pop_debug_scope(&self) {
        self.debug_scopes.borrow_mut().pop();
    }

    pub(crate) fn get_debug_scope(&self) -> Option<DIScope<'ctx>> {
        self.debug_scopes.borrow().last().copied()
    }
}

impl Codegen<'_> {
    pub(crate) fn set_span(&self, span: Span) {
        let location = self.di.builder.create_debug_location(
            self.context,
            span.start().line as u32,
            span.start().column as u32,
            self.di.get_debug_scope().unwrap(),
            None,
        );
        self.builder.set_current_debug_location(location);
    }
}
