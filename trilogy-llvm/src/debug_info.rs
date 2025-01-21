use inkwell::{
    debug_info::{
        AsDIScope, DIBasicType, DICompileUnit, DILocation, DIScope, DISubroutineType,
        DWARFEmissionKind, DWARFSourceLanguage, DebugInfoBuilder,
    },
    llvm_sys::debuginfo::LLVMDIFlagPublic,
    module::Module,
};
use source_span::Span;
use std::{cell::RefCell, rc::Rc};

use crate::codegen::Codegen;

pub(crate) struct DebugInfo<'ctx> {
    pub(crate) builder: DebugInfoBuilder<'ctx>,
    pub(crate) unit: DICompileUnit<'ctx>,
    pub(crate) debug_scopes: Rc<RefCell<Vec<DIScope<'ctx>>>>,
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
            debug_scopes: Rc::new(RefCell::new(vec![unit.as_debug_info_scope()])),
        }
    }

    #[inline(always)]
    pub(crate) fn validate(&self) {
        assert_eq!(self.debug_scopes.borrow().len(), 1);
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

    pub(crate) fn push_debug_scope(&self, scope: DIScope<'ctx>) -> DIScopeGuard<'ctx> {
        let mut scopes = self.debug_scopes.borrow_mut();
        scopes.push(scope);
        DIScopeGuard(self.debug_scopes.clone(), scopes.len())
    }

    pub(crate) fn push_block_scope(&self, span: Span) -> DIScopeGuard<'ctx> {
        let scope = self.get_debug_scope().unwrap();
        let block = self.builder.create_lexical_block(
            scope,
            self.unit.get_file(),
            span.start().line as u32,
            span.start().column as u32,
        );
        let mut scopes = self.debug_scopes.borrow_mut();
        scopes.push(block.as_debug_info_scope());
        DIScopeGuard(self.debug_scopes.clone(), scopes.len())
    }

    pub(crate) fn get_debug_scope(&self) -> Option<DIScope<'ctx>> {
        self.debug_scopes.borrow().last().copied()
    }
}

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn set_span(&self, span: Span) -> Option<DILocation<'ctx>> {
        let prev = self.builder.get_current_debug_location();
        let location = self.di.builder.create_debug_location(
            self.context,
            span.start().line as u32 + 1,
            span.start().column as u32 + 1,
            self.di.get_debug_scope().unwrap(),
            None,
        );
        self.builder.set_current_debug_location(location);
        prev
    }
}

pub(crate) struct DIScopeGuard<'ctx>(Rc<RefCell<Vec<DIScope<'ctx>>>>, usize);

impl Drop for DIScopeGuard<'_> {
    fn drop(&mut self) {
        let mut scopes = self.0.borrow_mut();
        assert_eq!(scopes.len(), self.1);
        scopes.pop();
    }
}
