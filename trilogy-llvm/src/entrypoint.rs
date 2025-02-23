use crate::{TrilogyValue, codegen::Codegen};
use inkwell::{
    AddressSpace,
    attributes::{Attribute, AttributeLoc},
    debug_info::AsDIScope,
    execution_engine::ExecutionEngine,
    llvm_sys::debuginfo::LLVMDIFlagPublic,
    memory_buffer::MemoryBuffer,
    module::Module,
};
use std::rc::Rc;

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn compile_standalone(&self, entrymodule: &str, entrypoint: &str) {
        let span = self
            .modules
            .get(entrymodule)
            .unwrap()
            .definitions()
            .iter()
            .find(|def| def.name().and_then(|id| id.name()) == Some(entrypoint))
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
        self.set_current_definition("main".to_owned(), span);
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
        let output = self.call_main(main);
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
        let output_ptr = self.module.add_global(self.value_type(), None, "output");
        self.execution_engine
            .add_global_mapping(&output_ptr, output as usize);
        let main_wrapper = self.module.add_function(
            "main",
            self.context.void_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
            None,
        );
        main_wrapper.add_attribute(
            AttributeLoc::Function,
            self.context
                .create_enum_attribute(Attribute::get_named_enum_kind_id("naked"), 1),
        );
        main_wrapper.add_attribute(
            AttributeLoc::Param(0),
            self.context.create_type_attribute(
                Attribute::get_named_enum_kind_id("sret"),
                self.value_type().into(),
            ),
        );

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
        let return_value = self.call_main(main);
        self.builder
            .build_store(output_ptr.as_pointer_value(), return_value)
            .unwrap();
        self.builder.build_return(None).unwrap();
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
                let bytes = atom.as_bytes();
                let string = self.module.add_global(
                    self.context.i8_type().array_type(bytes.len() as u32),
                    None,
                    "",
                );
                string.set_initializer(&self.context.const_string(bytes, false));
                self.string_value_type().const_named_struct(&[
                    self.context
                        .i64_type()
                        .const_int(bytes.len() as u64, false)
                        .into(),
                    string.as_pointer_value().into(),
                ])
            })
            .collect();
        atom_registry.set_initializer(&self.string_value_type().const_array(&atom_table));
    }

    pub(crate) fn finish(self) -> (Module<'ctx>, ExecutionEngine<'ctx>) {
        self.build_atom_registry();

        let core =
            MemoryBuffer::create_from_memory_range(include_bytes!("../core/core.bc"), "core");
        let core = Module::parse_bitcode_from_buffer(&core, self.context).unwrap();
        self.module.link_in_module(core).unwrap();
        self.di.builder.finalize();
        (Rc::into_inner(self.module).unwrap(), self.execution_engine)
    }
}
