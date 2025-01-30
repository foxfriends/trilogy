use inkwell::{
    debug_info::{
        AsDIScope, DIBasicType, DICompileUnit, DILexicalBlock, DILocation, DIScope, DISubprogram,
        DISubroutineType, DWARFEmissionKind, DWARFSourceLanguage, DebugInfoBuilder,
    },
    llvm_sys::debuginfo::LLVMDIFlagPublic,
    module::Module,
    values::FunctionValue,
};
use source_span::Span;
use std::{cell::RefCell, path::PathBuf, rc::Rc};
use url::Url;

use crate::codegen::Codegen;

pub(crate) struct DebugInfo<'ctx> {
    pub(crate) builder: DebugInfoBuilder<'ctx>,
    pub(crate) unit: DICompileUnit<'ctx>,
    debug_scopes: Rc<RefCell<Vec<DebugScope<'ctx>>>>,
}

#[derive(Clone, Copy)]
pub(crate) enum DebugScope<'ctx> {
    Unit(DICompileUnit<'ctx>),
    Subprogram(DISubprogram<'ctx>),
    LexicalBlock(DILexicalBlock<'ctx>, u32, u32),
}

impl<'ctx> DebugScope<'ctx> {
    fn as_debug_info_scope(&self) -> DIScope<'ctx> {
        match self {
            Self::Unit(scope) => scope.as_debug_info_scope(),
            Self::Subprogram(scope) => scope.as_debug_info_scope(),
            Self::LexicalBlock(scope, ..) => scope.as_debug_info_scope(),
        }
    }
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
            debug_scopes: Rc::new(RefCell::new(vec![DebugScope::Unit(unit)])),
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

    pub(crate) fn continuation_di_type(&self) -> DISubroutineType<'ctx> {
        self.builder.create_subroutine_type(
            self.unit.get_file(),
            Some(self.value_di_type().as_type()),
            &[self.value_di_type().as_type(); 5],
            LLVMDIFlagPublic,
        )
    }

    pub(crate) fn closure_di_type(&self, arity: usize) -> DISubroutineType<'ctx> {
        self.procedure_di_type(arity + 1)
    }

    pub(crate) fn push_subprogram(&self, scope: DISubprogram<'ctx>) {
        let mut scopes = self.debug_scopes.borrow_mut();
        scopes.push(DebugScope::Subprogram(scope));
    }

    pub(crate) fn push_block_scope(&self, span: Span) {
        let line = span.start().line as u32 + 1;
        let column = span.start().column as u32 + 1;
        let scope = self.get_debug_scope().unwrap();
        let block = self
            .builder
            .create_lexical_block(scope, self.unit.get_file(), line, column);
        let mut scopes = self.debug_scopes.borrow_mut();
        scopes.push(DebugScope::LexicalBlock(block, line, column));
    }

    pub(crate) fn pop_scope(&self) {
        self.debug_scopes.borrow_mut().pop();
    }

    pub(crate) fn get_debug_scope(&self) -> Option<DIScope<'ctx>> {
        self.debug_scopes
            .borrow()
            .last()
            .map(|s| s.as_debug_info_scope())
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

    pub(crate) fn transfer_debug_info(&self, function: FunctionValue<'ctx>) {
        let mut new_scopes = self.di.debug_scopes.borrow().clone();
        assert!(matches!(new_scopes[0], DebugScope::Unit(..)));
        assert!(matches!(new_scopes[1], DebugScope::Subprogram(..)));
        new_scopes[1] = DebugScope::Subprogram(function.get_subprogram().unwrap());
        for i in 2..new_scopes.len() {
            new_scopes[i] = match &new_scopes[i] {
                DebugScope::Unit(..) | DebugScope::Subprogram(..) => {
                    panic!("cannot have multiple units or subprograms")
                }
                DebugScope::LexicalBlock(.., line, column) => {
                    let scope = new_scopes[i - 1].as_debug_info_scope();
                    DebugScope::LexicalBlock(
                        self.di.builder.create_lexical_block(
                            scope,
                            self.di.unit.get_file(),
                            *line,
                            *column,
                        ),
                        *line,
                        *column,
                    )
                }
            }
        }
        let location = self.builder.get_current_debug_location().unwrap();
        let new_location = self.di.builder.create_debug_location(
            self.context,
            location.get_line(),
            location.get_column(),
            new_scopes.last().unwrap().as_debug_info_scope(),
            None,
        );
        *self.di.debug_scopes.borrow_mut() = new_scopes;
        self.builder.set_current_debug_location(new_location);
    }
}
