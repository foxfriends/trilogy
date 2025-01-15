use crate::{codegen::Codegen, scope::Scope};
use inkwell::{values::FunctionValue, IntPredicate};

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn atom_rt(&self) {
        self.build_lookup_table();
        self.define_lookup_const();
    }

    pub(crate) fn import_atom_rt(&self) {
        self.lookup_const();
    }

    fn build_lookup_table(&self) {
        let atoms = self.atoms.borrow();
        let mut atoms_vec: Vec<_> = atoms.iter().collect();
        atoms_vec.sort_by_key(|(_, s)| **s);
        let atom_strings = self.module.add_global(
            self.string_value_type().array_type(atoms_vec.len() as u32),
            None,
            "atom.strings",
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
        atom_strings.set_initializer(&self.string_value_type().const_array(&atom_table));
    }

    pub(crate) fn lookup_const(&self) -> FunctionValue<'ctx> {
        if let Some(printf) = self.module.get_function("trilogy:atom/rt::lookup_const") {
            return printf;
        }
        self.add_function("trilogy:atom/rt::lookup_const", true)
    }

    fn define_lookup_const(&self) {
        let atoms_len = self.atoms.borrow().len();
        let function = self.add_function("trilogy:atom/rt::lookup_const", true);
        let scope = Scope::begin(function);
        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);
        let atom = function.get_nth_param(1).unwrap().into_pointer_value();
        let atom = self.untag_atom(atom);
        let is_const = self
            .builder
            .build_int_compare(
                IntPredicate::ULT,
                atom,
                self.context.i64_type().const_int(atoms_len as u64, false),
                "",
            )
            .unwrap();

        let return_const = self.context.append_basic_block(function, "return_const");
        let return_unit = self.context.append_basic_block(function, "return_unit");

        self.builder
            .build_conditional_branch(is_const, return_const, return_unit)
            .unwrap();

        self.builder.position_at_end(return_const);
        let table = self.module.get_global("atom.strings").unwrap();
        let string = unsafe {
            self.builder
                .build_gep(
                    self.string_value_type().array_type(atoms_len as u32),
                    table.as_pointer_value(),
                    &[atom],
                    "",
                )
                .unwrap()
        };
        let string = self.string_value(string);
        self.builder.build_store(scope.sret(), string).unwrap();
        self.builder.build_return(None).unwrap();

        self.builder.position_at_end(return_unit);
        self.builder
            .build_store(scope.sret(), self.unit_const())
            .unwrap();
        self.builder.build_return(None).unwrap();
    }
}
