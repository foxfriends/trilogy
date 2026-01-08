use crate::TAIL_CALL_CONV;
use crate::{TrilogyValue, codegen::Codegen};
use inkwell::debug_info::AsDIScope;
use inkwell::execution_engine::ExecutionEngine;
use inkwell::llvm_sys::LLVMCallConv;
use inkwell::llvm_sys::debuginfo::LLVMDIFlagPublic;
use inkwell::memory_buffer::MemoryBuffer;
use inkwell::module::Module;
use std::rc::Rc;

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn compile_standalone(&self, entrymodule: &str, entrypoint: &str) {
        let span = self
            .modules
            .get(entrymodule)
            .unwrap()
            .definitions()
            .iter()
            .find(|def| def.name().map(|id| id.name()) == Some(entrypoint))
            .map(|def| def.span)
            .unwrap_or_default();

        let main_wrapper =
            self.module
                .add_function("main", self.context.void_type().fn_type(&[], false), None);
        let main_scope = self.di.builder.create_function(
            self.di.unit.get_file().as_debug_info_scope(),
            "main",
            None,
            self.di.unit.get_file(),
            span.start().line as u32 + 1,
            self.di.continuation_di_type(),
            true,
            true,
            span.start().line as u32 + 1,
            LLVMDIFlagPublic,
            false,
        );
        main_wrapper.set_subprogram(main_scope);
        let metadata = self.build_callable_data(entrymodule, "#entrypoint", 0, span, None);
        self.set_current_definition("main".to_owned(), "main".to_owned(), span, metadata, None);
        self.di.push_subprogram(main_scope);
        self.di.push_block_scope(span);
        self.set_span(span);
        let basic_block = self.context.append_basic_block(main_wrapper, "entry");

        self.builder.position_at_end(basic_block);

        // Reference main
        let main_accessor = self
            .module
            .get_function(&format!("{entrymodule}::{entrypoint}"))
            .unwrap();
        let main = self.allocate_value("main");
        self.call_internal(main, main_accessor, &[]);

        // Call main
        let output = self.call_main(main, &[], LLVMCallConv::LLVMFastCallConv as u32);
        _ = self.exit(output);
        self.close_continuation();
        self.di.pop_scope();
        self.di.pop_scope();
    }

    pub(crate) fn compile_embedded(
        &self,
        entrymodule: &str,
        entrypoint: &str,
        output: *mut TrilogyValue,
    ) {
        let span = self
            .modules
            .get(entrymodule)
            .unwrap()
            .definitions()
            .iter()
            .find(|def| def.name().map(|id| id.name()) == Some(entrypoint))
            .map(|def| def.span)
            .unwrap_or_default();
        let output_ptr = self.module.add_global(self.value_type(), None, "output");
        self.execution_engine
            .add_global_mapping(&output_ptr, output as usize);

        let main_wrapper =
            self.module
                .add_function("main", self.context.void_type().fn_type(&[], false), None);
        let main_scope = self.di.builder.create_function(
            self.di.unit.get_file().as_debug_info_scope(),
            "main",
            None,
            self.di.unit.get_file(),
            span.start().line as u32 + 1,
            self.di.continuation_di_type(),
            true,
            true,
            span.start().line as u32 + 1,
            LLVMDIFlagPublic,
            false,
        );
        main_wrapper.set_subprogram(main_scope);
        let metadata = self.build_callable_data(entrymodule, "#entrypoint", 0, span, None);
        self.set_current_definition("main".to_owned(), "main".to_owned(), span, metadata, None);
        self.di.push_subprogram(main_scope);
        self.di.push_block_scope(span);
        self.set_span(span);
        let basic_block = self.context.append_basic_block(main_wrapper, "entry");

        self.builder.position_at_end(basic_block);
        // Reference main
        let main_accessor = self
            .module
            .get_function(&format!("{entrymodule}::{entrypoint}"))
            .unwrap();
        let main = self.allocate_value("main");
        self.call_internal(main, main_accessor, &[]);

        // Call main
        let return_pointer = self.call_main(main, &[], LLVMCallConv::LLVMFastCallConv as u32);
        let return_value = self
            .builder
            .build_load(self.value_type(), return_pointer, "")
            .unwrap();
        self.builder
            .build_store(output_ptr.as_pointer_value(), return_value)
            .unwrap();
        self.builder.build_return(None).unwrap();
        self.close_continuation();
        self.di.pop_scope();
        self.di.pop_scope();
    }

    pub(crate) fn compile_test_entrypoint(&self, test_accessor_names: &[&str]) {
        let span = source_span::Span::default();
        let main_wrapper =
            self.module
                .add_function("main", self.context.void_type().fn_type(&[], false), None);
        let main_scope = self.di.builder.create_function(
            self.di.unit.get_file().as_debug_info_scope(),
            "main",
            None,
            self.di.unit.get_file(),
            span.start().line as u32 + 1,
            self.di.continuation_di_type(),
            true,
            true,
            span.start().line as u32 + 1,
            LLVMDIFlagPublic,
            false,
        );
        main_wrapper.set_subprogram(main_scope);
        let metadata = self.build_callable_data("trilogy", "#entrypoint", 0, span, None);
        self.set_current_definition("main".to_owned(), "main".to_owned(), span, metadata, None);
        self.di.push_subprogram(main_scope);
        self.di.push_block_scope(span);
        self.set_span(span);
        let basic_block = self.context.append_basic_block(main_wrapper, "entry");
        self.builder.position_at_end(basic_block);

        let test_manifest = self.allocate_value("tests");
        let test_array =
            self.trilogy_array_init_cap(test_manifest, test_accessor_names.len(), "tests_array");
        for name in test_accessor_names {
            let accessor = self.import_accessor(name);
            let name_value = self.allocate_value("");
            self.string_const(name_value, name);
            let function_value = self.allocate_value("");
            self.call_internal(function_value, accessor, &[]);
            let test_tuple = self.allocate_value("");
            self.trilogy_tuple_init_new(test_tuple, name_value, function_value);
            self.trilogy_array_push(test_array, test_tuple);
        }

        let main = self.test_main();
        self.call_main(
            main,
            &[self
                .builder
                .build_load(self.value_type(), test_manifest, "")
                .unwrap()
                .into()],
            TAIL_CALL_CONV,
        );
        self.builder.build_return(None).unwrap();
        self.close_continuation();
        self.di.pop_scope();
        self.di.pop_scope();
    }

    fn build_atom_registry(&self) {
        let atoms = self.atoms.borrow();
        let mut atoms_vec: Vec<_> = atoms.iter().collect();
        atoms_vec.sort_by_key(|(_, s)| **s);
        let atom_registry_sz =
            self.module
                .add_global(self.context.i64_type(), None, "atom_registry_sz");
        atom_registry_sz.set_initializer(
            &self
                .context
                .i64_type()
                .const_int(atoms_vec.len() as u64, false),
        );
        let atom_registry = self.module.add_global(
            self.string_value_type().array_type(atoms_vec.len() as u32),
            None,
            "atom_registry",
        );
        let atom_table: Vec<_> = atoms_vec
            .into_iter()
            .map(|(atom, _)| {
                let string = self.global_c_string(atom, true);
                self.string_value_type().const_named_struct(&[
                    self.context
                        .i64_type()
                        .const_int(atom.len() as u64, false)
                        .into(),
                    string.as_pointer_value().into(),
                ])
            })
            .collect();
        atom_registry.set_initializer(&self.string_value_type().const_array(&atom_table));
    }

    pub(crate) fn finish(self) -> (Module<'ctx>, ExecutionEngine<'ctx>) {
        self.build_atom_registry();

        let core = MemoryBuffer::create_from_memory_range(
            include_bytes!("../core/trilogy_core.bc"),
            "core",
        );
        log::debug!("parsing trilogy_core.bc");
        let core = Module::parse_bitcode_from_buffer(&core, self.context).unwrap();
        self.module.link_in_module(core).unwrap();
        self.di.builder.finalize();
        (Rc::into_inner(self.module).unwrap(), self.execution_engine)
    }
}
