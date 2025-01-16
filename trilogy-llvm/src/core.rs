use crate::codegen::Codegen;
use inkwell::{module::Linkage, values::FunctionValue, AddressSpace};

macro_rules! declare_procedure {
    ($name:ident, $arity:literal) => {
        pub(crate) fn $name(&self) -> FunctionValue<'ctx> {
            if let Some(func) = self.module.get_function(stringify!($name)) {
                return func;
            }
            self.add_procedure(stringify!($name), $arity, true)
        }
    };
}

impl<'ctx> Codegen<'ctx> {
    /// Declares the procedures exported from `trilogy:core`.
    pub(crate) fn import_core(&self) {
        self.trilogy_structural_eq();
        self.trilogy_panic();
        self.trilogy_print();
        self.trilogy_exit();
        self.trilogy_lookup_atom();
    }

    /// Declares the low level functions for use in the language internals.
    pub(crate) fn import_internal(&self) {
        self.module.add_function(
            "untag_unit",
            self.context.void_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
            Some(Linkage::AvailableExternally),
        );
        self.module.add_function(
            "untag_bool",
            self.context.bool_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
            Some(Linkage::AvailableExternally),
        );
        self.module.add_function(
            "untag_atom",
            self.context.i64_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
            Some(Linkage::AvailableExternally),
        );
        self.module.add_function(
            "untag_character",
            self.context.i32_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
            Some(Linkage::AvailableExternally),
        );
        self.module.add_function(
            "untag_string",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
            Some(Linkage::AvailableExternally),
        );
        self.module.add_function(
            "untag_integer",
            self.context.i64_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
            Some(Linkage::AvailableExternally),
        );
        self.module.add_function(
            "untag_bits",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
            Some(Linkage::AvailableExternally),
        );
        self.module.add_function(
            "untag_struct",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
            Some(Linkage::AvailableExternally),
        );
        self.module.add_function(
            "untag_tuple",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
            Some(Linkage::AvailableExternally),
        );
        self.module.add_function(
            "untag_array",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
            Some(Linkage::AvailableExternally),
        );
        self.module.add_function(
            "untag_set",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
            Some(Linkage::AvailableExternally),
        );
        self.module.add_function(
            "untag_record",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
            Some(Linkage::AvailableExternally),
        );
        self.module.add_function(
            "untag_callable",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
            Some(Linkage::AvailableExternally),
        );
    }

    declare_procedure!(trilogy_panic, 1);
    declare_procedure!(trilogy_exit, 1);
    declare_procedure!(trilogy_structural_eq, 2);
    declare_procedure!(trilogy_print, 1);
    declare_procedure!(trilogy_lookup_atom, 1);
}
