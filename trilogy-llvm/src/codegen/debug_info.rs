use inkwell::AddressSpace;
use inkwell::builder::Builder;
use inkwell::debug_info::{
    AsDIScope, DICompileUnit, DICompositeType, DIDerivedType, DILexicalBlock, DILocation, DIScope,
    DISubprogram, DISubroutineType, DWARFEmissionKind, DWARFSourceLanguage, DebugInfoBuilder,
};
use inkwell::llvm_sys::debuginfo::LLVMDIFlagPublic;
use inkwell::module::Module;
use inkwell::values::PointerValue;
use source_span::Span;
use std::{cell::RefCell, path::PathBuf, rc::Rc};
use url::Url;

use crate::codegen::Codegen;

pub(crate) struct DebugInfo<'ctx> {
    pub(crate) builder: DebugInfoBuilder<'ctx>,
    pub(crate) unit: DICompileUnit<'ctx>,
    pub(super) debug_scopes: Rc<RefCell<Vec<Vec<DebugScope<'ctx>>>>>,

    value_type: DICompositeType<'ctx>,
    value_pointer_type: DIDerivedType<'ctx>,
    continuation_type: DISubroutineType<'ctx>,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum DebugScope<'ctx> {
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

        let value_type = builder.create_struct_type(
            unit.get_file().as_debug_info_scope(),
            "trilogy_value",
            unit.get_file(),
            0,
            72,
            0,
            LLVMDIFlagPublic,
            None,
            &[],
            0,
            None,
            "",
        );

        let value_pointer_type = builder.create_pointer_type(
            "trilogy_value",
            value_type.as_type(),
            0,
            0,
            AddressSpace::default(),
        );

        let continuation_type = builder.create_subroutine_type(
            unit.get_file(),
            None,
            &[value_pointer_type.as_type(); 7],
            LLVMDIFlagPublic,
        );

        DebugInfo {
            builder,
            unit,
            debug_scopes: Rc::new(RefCell::new(vec![])),
            value_type,
            value_pointer_type,
            continuation_type,
        }
    }

    pub(super) fn create_function(
        &self,
        name: &str,
        linkage_name: &str,
        di_type: DISubroutineType<'ctx>,
        span: Span,
        is_local_to_unit: bool,
        is_definition: bool,
    ) -> DISubprogram<'ctx> {
        self.builder.create_function(
            self.unit.get_file().as_debug_info_scope(),
            name,
            Some(linkage_name),
            self.unit.get_file(),
            span.start().line as u32 + 1,
            di_type,
            is_local_to_unit,
            is_definition,
            span.start().line as u32 + 1,
            LLVMDIFlagPublic,
            false,
        )
    }

    pub(crate) fn value_di_type(&self) -> DICompositeType<'ctx> {
        self.value_type
    }

    pub(crate) fn procedure_di_type(&self, arity: usize) -> DISubroutineType<'ctx> {
        self.builder.create_subroutine_type(
            self.unit.get_file(),
            None,
            &vec![self.value_pointer_type.as_type(); arity + 5],
            LLVMDIFlagPublic,
        )
    }

    pub(crate) fn continuation_di_type(&self) -> DISubroutineType<'ctx> {
        self.continuation_type
    }

    pub(crate) fn closure_di_type(&self, arity: usize) -> DISubroutineType<'ctx> {
        self.procedure_di_type(arity + 1)
    }

    pub(crate) fn push_subprogram(&self, scope: DISubprogram<'ctx>) {
        let mut scopes = self.debug_scopes.borrow_mut();
        scopes.push(vec![
            DebugScope::Unit(self.unit),
            DebugScope::Subprogram(scope),
        ]);
    }

    pub(crate) fn push_block_scope(&self, span: Span) {
        let line = span.start().line as u32 + 1;
        let column = span.start().column as u32 + 1;
        let scope = self.get_debug_scope();
        let block = self
            .builder
            .create_lexical_block(scope, self.unit.get_file(), line, column);
        let mut scopes = self.debug_scopes.borrow_mut();
        scopes
            .last_mut()
            .unwrap()
            .push(DebugScope::LexicalBlock(block, line, column));
    }

    pub(crate) fn pop_scope(&self) {
        let mut scope_stacks = self.debug_scopes.borrow_mut();
        if matches!(
            scope_stacks.last_mut().unwrap().pop().unwrap(),
            DebugScope::Subprogram(..)
        ) {
            scope_stacks.pop();
        }
    }

    pub(crate) fn get_debug_scope(&self) -> DIScope<'ctx> {
        self.debug_scopes
            .borrow()
            .last()
            .unwrap()
            .last()
            .unwrap()
            .as_debug_info_scope()
    }

    pub(crate) fn describe_variable(
        &self,
        variable: PointerValue<'ctx>,
        name: &str,
        span: Span,
        builder: &Builder<'ctx>,
        function: DISubprogram<'ctx>,
        location: DILocation<'ctx>,
    ) {
        let di_variable = self.builder.create_auto_variable(
            function.as_debug_info_scope(),
            name,
            self.unit.get_file(),
            span.start().line as u32 + 1,
            self.value_di_type().as_type(),
            true,
            LLVMDIFlagPublic,
            0,
        );
        self.builder.insert_declare_at_end(
            variable,
            Some(di_variable),
            None,
            location,
            builder.get_insert_block().unwrap(),
        );
    }
}

impl<'ctx> Codegen<'ctx> {
    pub(super) fn create_debug_location(&self, span: Span) -> DILocation<'ctx> {
        self.di.builder.create_debug_location(
            self.context,
            span.start().line as u32 + 1,
            span.start().column as u32 + 1,
            self.di.get_debug_scope(),
            None,
        )
    }

    pub(crate) fn set_span(&self, span: Span) -> Option<DILocation<'ctx>> {
        let prev = self.builder.get_current_debug_location();
        let location = self.create_debug_location(span);
        self.builder.set_current_debug_location(location);
        prev
    }

    pub(crate) fn overwrite_debug_location(&self, location: DILocation<'ctx>) {
        let current_scope = self.di.get_debug_scope();
        if location.get_scope() == current_scope {
            self.builder.set_current_debug_location(location)
        } else {
            let new_location = self.di.builder.create_debug_location(
                self.context,
                location.get_line(),
                location.get_column(),
                current_scope,
                None,
            );
            self.builder.set_current_debug_location(new_location);
        }
    }

    pub(crate) fn transfer_debug_info(&self) {
        let function = self.get_function();
        let mut scope_stacks = self.di.debug_scopes.borrow_mut();
        let scopes = scope_stacks.last_mut().unwrap();
        let mut new_scopes = Vec::with_capacity(scopes.len());
        assert!(matches!(scopes[0], DebugScope::Unit(..)));
        assert!(matches!(scopes[1], DebugScope::Subprogram(..)));
        new_scopes.push(scopes[0]);
        new_scopes.push(DebugScope::Subprogram(function.get_subprogram().unwrap()));
        for i in 2..scopes.len() {
            new_scopes.push(match &scopes[i] {
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
            });
        }
        let location = self.builder.get_current_debug_location().unwrap();
        let new_location = self.di.builder.create_debug_location(
            self.context,
            location.get_line(),
            location.get_column(),
            new_scopes.last().unwrap().as_debug_info_scope(),
            None,
        );
        *scopes = new_scopes;
        self.builder.set_current_debug_location(new_location);
    }
}
