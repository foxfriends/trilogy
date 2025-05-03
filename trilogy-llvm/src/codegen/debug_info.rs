use inkwell::AddressSpace;
use inkwell::builder::Builder;
use inkwell::debug_info::{
    AsDIScope, DICompileUnit, DICompositeType, DIDerivedType, DILexicalBlock, DILocation, DIScope,
    DISubprogram, DISubroutineType, DWARFEmissionKind, DWARFSourceLanguage, DebugInfoBuilder,
};
use inkwell::execution_engine::ExecutionEngine;
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
    pub(super) debug_scopes: Rc<RefCell<Vec<DebugScope<'ctx>>>>,

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
    pub(crate) fn new(module: &Module<'ctx>, name: &str, ee: &ExecutionEngine) -> Self {
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
            DWARFSourceLanguage::C11,
            &filename,
            &directory,
            concat!("trilogy ", env!("CARGO_PKG_VERSION")),
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

        let ptr_size = ee.get_target_data().get_pointer_byte_size(None) as u64 * 8;

        // I think this is the LLVMDwarfTypeEncodings?
        // https://github.com/llvm-mirror/llvm/blob/2c4ca6832fa6b306ee6a7010bfb80a3f2596f824/include/llvm/BinaryFormat/Dwarf.def#L702-L723

        let tag_type = builder
            .create_basic_type(
                "value_tag",
                8,
                // Unsigned char
                0x8,
                LLVMDIFlagPublic,
            )
            .unwrap();

        let bool_type = builder
            .create_basic_type("boolean", 8, 0x2, LLVMDIFlagPublic)
            .unwrap();

        let size_t = builder
            .create_basic_type("size_t", ptr_size, 0x7, LLVMDIFlagPublic)
            .unwrap();
        let digit_t = builder
            .create_basic_type("digit_t", 32, 0x7, LLVMDIFlagPublic)
            .unwrap();

        let string_type = builder.create_struct_type(
            unit.get_file().as_debug_info_scope(),
            "string",
            unit.get_file(),
            0,
            ptr_size * 2,
            0,
            LLVMDIFlagPublic,
            None,
            &[
                builder
                    .create_member_type(
                        unit.get_file().as_debug_info_scope(),
                        "len",
                        unit.get_file(),
                        0,
                        ptr_size,
                        0,
                        0,
                        LLVMDIFlagPublic,
                        builder
                            .create_basic_type("size_t", ptr_size, 0x7, LLVMDIFlagPublic)
                            .unwrap()
                            .as_type(),
                    )
                    .as_type(),
                builder
                    .create_member_type(
                        unit.get_file().as_debug_info_scope(),
                        "contents",
                        unit.get_file(),
                        0,
                        ptr_size,
                        0,
                        ptr_size,
                        LLVMDIFlagPublic,
                        builder
                            .create_pointer_type(
                                "char*",
                                builder
                                    .create_basic_type("char", 8, 0x8, LLVMDIFlagPublic)
                                    .unwrap()
                                    .as_type(),
                                ptr_size,
                                0,
                                AddressSpace::default(),
                            )
                            .as_type(),
                    )
                    .as_type(),
            ],
            0,
            None,
            "",
        );

        let bigint_size = ptr_size * 3;
        let bigint_type = builder.create_struct_type(
            unit.get_file().as_debug_info_scope(),
            "bigint",
            unit.get_file(),
            0,
            bigint_size,
            0,
            LLVMDIFlagPublic,
            None,
            &[
                builder
                    .create_member_type(
                        unit.get_file().as_debug_info_scope(),
                        "capacity",
                        unit.get_file(),
                        0,
                        ptr_size,
                        0,
                        0,
                        LLVMDIFlagPublic,
                        size_t.as_type(),
                    )
                    .as_type(),
                builder
                    .create_member_type(
                        unit.get_file().as_debug_info_scope(),
                        "length",
                        unit.get_file(),
                        0,
                        ptr_size,
                        0,
                        ptr_size,
                        LLVMDIFlagPublic,
                        size_t.as_type(),
                    )
                    .as_type(),
                builder
                    .create_member_type(
                        unit.get_file().as_debug_info_scope(),
                        "contents",
                        unit.get_file(),
                        0,
                        ptr_size,
                        0,
                        ptr_size * 2,
                        LLVMDIFlagPublic,
                        builder
                            .create_union_type(
                                unit.get_file().as_debug_info_scope(),
                                "",
                                unit.get_file(),
                                0,
                                ptr_size,
                                0,
                                LLVMDIFlagPublic,
                                &[
                                    builder
                                        .create_member_type(
                                            unit.get_file().as_debug_info_scope(),
                                            "digits",
                                            unit.get_file(),
                                            0,
                                            8,
                                            0,
                                            0,
                                            LLVMDIFlagPublic,
                                            builder
                                                .create_pointer_type(
                                                    "",
                                                    digit_t.as_type(),
                                                    ptr_size,
                                                    0,
                                                    AddressSpace::default(),
                                                )
                                                .as_type(),
                                        )
                                        .as_type(),
                                    builder
                                        .create_member_type(
                                            unit.get_file().as_debug_info_scope(),
                                            "value",
                                            unit.get_file(),
                                            0,
                                            32,
                                            0,
                                            0,
                                            LLVMDIFlagPublic,
                                            digit_t.as_type(),
                                        )
                                        .as_type(),
                                ],
                                0,
                                "",
                            )
                            .as_type(),
                    )
                    .as_type(),
            ],
            0,
            None,
            "",
        );

        let rational_size = ptr_size + bigint_size * 2;

        let rational_type = builder.create_struct_type(
            unit.get_file().as_debug_info_scope(),
            "number",
            unit.get_file(),
            0,
            rational_size,
            0,
            LLVMDIFlagPublic,
            None,
            &[
                builder
                    .create_member_type(
                        unit.get_file().as_debug_info_scope(),
                        "is_negative",
                        unit.get_file(),
                        0,
                        8,
                        0,
                        0,
                        LLVMDIFlagPublic,
                        bool_type.as_type(),
                    )
                    .as_type(),
                builder
                    .create_member_type(
                        unit.get_file().as_debug_info_scope(),
                        "numer",
                        unit.get_file(),
                        0,
                        bigint_size,
                        0,
                        ptr_size,
                        LLVMDIFlagPublic,
                        bigint_type.as_type(),
                    )
                    .as_type(),
                builder
                    .create_member_type(
                        unit.get_file().as_debug_info_scope(),
                        "denom",
                        unit.get_file(),
                        0,
                        bigint_size,
                        0,
                        bigint_size + ptr_size,
                        LLVMDIFlagPublic,
                        bigint_type.as_type(),
                    )
                    .as_type(),
            ],
            0,
            None,
            "",
        );

        let number_type = builder.create_struct_type(
            unit.get_file().as_debug_info_scope(),
            "number",
            unit.get_file(),
            0,
            rational_size * 2,
            0,
            LLVMDIFlagPublic,
            None,
            &[
                builder
                    .create_member_type(
                        unit.get_file().as_debug_info_scope(),
                        "re",
                        unit.get_file(),
                        0,
                        rational_size,
                        0,
                        0,
                        LLVMDIFlagPublic,
                        rational_type.as_type(),
                    )
                    .as_type(),
                builder
                    .create_member_type(
                        unit.get_file().as_debug_info_scope(),
                        "im",
                        unit.get_file(),
                        0,
                        rational_size,
                        0,
                        rational_size,
                        LLVMDIFlagPublic,
                        rational_type.as_type(),
                    )
                    .as_type(),
            ],
            0,
            None,
            "",
        );

        let any_value_type = builder.create_union_type(
            unit.get_file().as_debug_info_scope(),
            "value_payload",
            unit.get_file(),
            0,
            64,
            0,
            LLVMDIFlagPublic,
            &[
                builder
                    .create_member_type(
                        unit.get_file().as_debug_info_scope(),
                        "undefined",
                        unit.get_file(),
                        0,
                        0,
                        0,
                        0,
                        LLVMDIFlagPublic,
                        builder
                            .create_basic_type("undefined", 0, 0x07, LLVMDIFlagPublic)
                            .unwrap()
                            .as_type(),
                    )
                    .as_type(),
                builder
                    .create_member_type(
                        unit.get_file().as_debug_info_scope(),
                        "unit",
                        unit.get_file(),
                        0,
                        0,
                        0,
                        0,
                        LLVMDIFlagPublic,
                        builder
                            .create_basic_type("unit", 0, 0x07, LLVMDIFlagPublic)
                            .unwrap()
                            .as_type(),
                    )
                    .as_type(),
                builder
                    .create_member_type(
                        unit.get_file().as_debug_info_scope(),
                        "boolean",
                        unit.get_file(),
                        0,
                        8,
                        0,
                        0,
                        LLVMDIFlagPublic,
                        bool_type.as_type(),
                    )
                    .as_type(),
                builder
                    .create_member_type(
                        unit.get_file().as_debug_info_scope(),
                        "character",
                        unit.get_file(),
                        0,
                        64,
                        0,
                        0,
                        LLVMDIFlagPublic,
                        builder
                            .create_basic_type("character", 64, 0x7, LLVMDIFlagPublic)
                            .unwrap()
                            .as_type(),
                    )
                    .as_type(),
                builder
                    .create_member_type(
                        unit.get_file().as_debug_info_scope(),
                        "string",
                        unit.get_file(),
                        0,
                        ptr_size,
                        0,
                        0,
                        LLVMDIFlagPublic,
                        builder
                            .create_pointer_type(
                                "string_ptr",
                                string_type.as_type(),
                                ptr_size,
                                0,
                                AddressSpace::default(),
                            )
                            .as_type(),
                    )
                    .as_type(),
                builder
                    .create_member_type(
                        unit.get_file().as_debug_info_scope(),
                        "number",
                        unit.get_file(),
                        0,
                        ptr_size,
                        0,
                        0,
                        LLVMDIFlagPublic,
                        builder
                            .create_pointer_type(
                                "number_ptr",
                                number_type.as_type(),
                                ptr_size,
                                0,
                                AddressSpace::default(),
                            )
                            .as_type(),
                    )
                    .as_type(),
            ],
            0,
            "",
        );

        let value_type = builder.create_struct_type(
            unit.get_file().as_debug_info_scope(),
            "value",
            unit.get_file(),
            0,
            128,
            0,
            LLVMDIFlagPublic,
            None,
            &[
                builder
                    .create_member_type(
                        unit.get_file().as_debug_info_scope(),
                        "tag",
                        unit.get_file(),
                        0,
                        8,
                        0,
                        0,
                        LLVMDIFlagPublic,
                        tag_type.as_type(),
                    )
                    .as_type(),
                builder
                    .create_member_type(
                        unit.get_file().as_debug_info_scope(),
                        "payload",
                        unit.get_file(),
                        0,
                        64,
                        0,
                        ptr_size,
                        LLVMDIFlagPublic,
                        any_value_type.as_type(),
                    )
                    .as_type(),
            ],
            0,
            None,
            "",
        );

        let value_pointer_type = builder.create_pointer_type(
            "value_ptr",
            value_type.as_type(),
            ptr_size,
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
            &vec![self.value_pointer_type.as_type(); arity + 7],
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
        *scopes = vec![DebugScope::Unit(self.unit), DebugScope::Subprogram(scope)];
    }

    pub(crate) fn push_block_scope(&self, span: Span) {
        let line = span.start().line as u32 + 1;
        let column = span.start().column as u32 + 1;
        let scope = self.get_debug_scope();
        let block = self
            .builder
            .create_lexical_block(scope, self.unit.get_file(), line, column);
        let mut scopes = self.debug_scopes.borrow_mut();
        scopes.push(DebugScope::LexicalBlock(block, line, column));
    }

    pub(crate) fn pop_scope(&self) {
        self.debug_scopes
            .borrow_mut()
            .pop()
            .expect("pop scope called too many times");
    }

    pub(crate) fn get_debug_scope(&self) -> DIScope<'ctx> {
        self.debug_scopes
            .borrow()
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
        let mut scopes = self.di.debug_scopes.borrow_mut();
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
