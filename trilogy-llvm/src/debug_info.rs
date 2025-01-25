use inkwell::{
    debug_info::{
        AsDIScope, DIBasicType, DICompileUnit, DILocation, DIScope, DISubroutineType,
        DWARFEmissionKind, DWARFSourceLanguage, DebugInfoBuilder,
    },
    llvm_sys::debuginfo::LLVMDIFlagPublic,
    module::Module,
};
use source_span::Span;
use std::{cell::RefCell, path::PathBuf, rc::Rc};
use url::Url;

use crate::codegen::Codegen;

pub(crate) struct DebugInfo<'ctx> {
    pub(crate) builder: DebugInfoBuilder<'ctx>,
    pub(crate) unit: DICompileUnit<'ctx>,
    pub(crate) debug_scopes: Rc<RefCell<Vec<DIScope<'ctx>>>>,
}

impl<'ctx> DebugInfo<'ctx> {
    pub(crate) fn new(module: &Module<'ctx>, name: &str) -> Self {
        let url = Url::parse(name).unwrap();
        let (filename, directory) = match url.scheme() {
            "file" => {
                let path: PathBuf = url.path().parse().unwrap();
                (
                    path.file_name().unwrap().to_string_lossy().into_owned(),
                    path.parent().unwrap().display().to_string(),
                )
            }
            "http" | "https" => {
                let path: PathBuf = url.path().parse().unwrap();
                (
                    path.file_name().unwrap().to_string_lossy().into_owned(),
                    path.parent().unwrap().display().to_string(),
                )
            }
            "trilogy" => (url.path().to_owned(), "/".to_owned()),
            _ => (name.to_owned(), "/".to_owned()),
        };
        let (builder, unit) = module.create_debug_info_builder(
            true,
            DWARFSourceLanguage::C,
            &filename,
            &directory,
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

    pub(crate) fn closure_di_type(&self, arity: usize) -> DISubroutineType<'ctx> {
        // TODO: the last parameter is NOT a value, but we say it is anyway :shrug:
        self.procedure_di_type(arity + 1)
    }

    pub(crate) fn push_debug_scope(&self, scope: DIScope<'ctx>) {
        let mut scopes = self.debug_scopes.borrow_mut();
        scopes.push(scope);
    }

    pub(crate) fn push_block_scope(&self, span: Span) {
        let scope = self.get_debug_scope().unwrap();
        let block = self.builder.create_lexical_block(
            scope,
            self.unit.get_file(),
            span.start().line as u32 + 1,
            span.start().column as u32 + 1,
        );
        let mut scopes = self.debug_scopes.borrow_mut();
        scopes.push(block.as_debug_info_scope());
    }

    pub(crate) fn pop_scope(&self) {
        self.debug_scopes.borrow_mut().pop();
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
