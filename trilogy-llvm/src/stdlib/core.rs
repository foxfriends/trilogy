use crate::codegen::Codegen;
use inkwell::{module::Linkage, values::FunctionValue, AddressSpace};

macro_rules! declare {
    ($name:ident, $arity:literal) => {
        pub(crate) fn $name(&self) -> FunctionValue<'ctx> {
            if let Some(func) = self
                .module
                .get_function(concat!("trilogy_", stringify!($name)))
            {
                return func;
            }
            self.add_procedure(concat!("trilogy_", stringify!($name)), $arity, true)
        }
    };
}

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn import_core(&self) {
        self.structural_eq();
        self.panic();
        self.printf();
        self.exit();

        self.module.add_function(
            "untag_unit",
            self.context.void_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
            Some(Linkage::External),
        );
        self.module.add_function(
            "untag_bool",
            self.context.bool_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
            Some(Linkage::External),
        );
        self.module.add_function(
            "untag_atom",
            self.context.i64_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
            Some(Linkage::External),
        );
        self.module.add_function(
            "untag_char",
            self.context.i32_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
            Some(Linkage::External),
        );
        self.module.add_function(
            "untag_string",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
            Some(Linkage::External),
        );
        self.module.add_function(
            "untag_integer",
            self.context.i64_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
            Some(Linkage::External),
        );
        self.module.add_function(
            "untag_bits",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
            Some(Linkage::External),
        );
        self.module.add_function(
            "untag_struct",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
            Some(Linkage::External),
        );
        self.module.add_function(
            "untag_tuple",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
            Some(Linkage::External),
        );
        self.module.add_function(
            "untag_array",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
            Some(Linkage::External),
        );
        self.module.add_function(
            "untag_set",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
            Some(Linkage::External),
        );
        self.module.add_function(
            "untag_record",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
            Some(Linkage::External),
        );
        self.module.add_function(
            "untag_callable",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
            Some(Linkage::External),
        );
    }

    declare!(panic, 1);
    declare!(exit, 1);
    declare!(structural_eq, 2);
    declare!(printf, 1);
}
