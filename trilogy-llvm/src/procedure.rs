use crate::{scope::Scope, Codegen};
use inkwell::{
    attributes::{Attribute, AttributeLoc},
    debug_info::AsDIScope,
    llvm_sys::debuginfo::LLVMDIFlagPublic,
    module::Linkage,
    values::FunctionValue,
    AddressSpace,
};
use source_span::Span;
use trilogy_ir::ir;

const MAIN_NAME: &str = "trilogy:::main";

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn declare_procedure(
        &self,
        name: &str,
        arity: usize,
        linkage: Linkage,
        is_extern: bool,
        span: Span,
    ) -> FunctionValue<'ctx> {
        let long_name = format!("{}::{}", self.location, name);

        let procedure_scope = self.di.builder.create_function(
            self.di.unit.as_debug_info_scope(),
            name,
            None,
            self.di.unit.get_file(),
            span.start().line as u32,
            self.di.procedure_di_type(arity),
            linkage == Linkage::External,
            !is_extern,
            0,
            LLVMDIFlagPublic,
            false,
        );

        let function = self.module.add_function(
            if name == "main" { MAIN_NAME } else { name },
            self.procedure_type(arity),
            Some(if is_extern {
                Linkage::External
            } else {
                Linkage::Private
            }),
        );
        function.set_subprogram(procedure_scope);

        if !is_extern {
            // Internal procedures first parameters are marked with `sret`. External procedures
            // just fill the first parameter by convention, but it's not actually `sret` cause it's just C.
            function.add_attribute(
                AttributeLoc::Param(0),
                self.context.create_type_attribute(
                    Attribute::get_named_enum_kind_id("sret"),
                    self.value_type().into(),
                ),
            );
            function.get_nth_param(0).unwrap().set_name("sretptr");
        }

        let accessor = self
            .module
            .add_function(&long_name, self.procedure_type(0), Some(linkage));
        accessor.add_attribute(
            AttributeLoc::Param(0),
            self.context.create_type_attribute(
                Attribute::get_named_enum_kind_id("sret"),
                self.value_type().into(),
            ),
        );
        let sret = accessor.get_nth_param(0).unwrap().into_pointer_value();
        let accessor_entry = self.context.append_basic_block(accessor, "entry");
        self.builder.position_at_end(accessor_entry);
        self.trilogy_callable_init_proc(
            sret,
            arity,
            self.context.ptr_type(AddressSpace::default()).const_zero(),
            function.as_global_value().as_pointer_value(),
        );
        self.builder.build_return(None).unwrap();

        function
    }

    pub(crate) fn import_procedure(
        &self,
        location: &str,
        definition: &ir::ProcedureDefinition,
    ) -> FunctionValue<'ctx> {
        let long_name = format!("{}::{}", location, definition.name);
        if let Some(function) = self.module.get_function(&long_name) {
            return function;
        }
        self.module
            .add_function(&long_name, self.procedure_type(0), Some(Linkage::External))
    }

    pub(crate) fn compile_procedure(&self, definition: &ir::ProcedureDefinition) {
        if definition.overloads.is_empty() {
            // There may be no overloads, indicating this is an externally defined procedure.
            return;
        }
        assert_eq!(definition.overloads.len(), 1);
        let procedure = &definition.overloads[0];
        let name = definition.name.to_string();
        let function = self
            .module
            .get_function(if name == "main" { MAIN_NAME } else { &name })
            .unwrap();

        let entry = self.context.append_basic_block(function, "entry");
        let no_match = self.context.append_basic_block(function, "no_match");
        let cleanup = self.context.append_basic_block(function, "cleanup");

        let mut scope = Scope::begin(function);

        self.di
            .push_debug_scope(function.get_subprogram().unwrap().as_debug_info_scope());
        scope.set_cleanup(cleanup);

        'body: {
            self.builder.position_at_end(entry);
            for (n, param) in procedure.parameters.iter().enumerate() {
                let value = function
                    .get_nth_param(n as u32 + 1)
                    .unwrap()
                    .into_pointer_value();
                if self
                    .compile_pattern_match(&mut scope, param, value, no_match)
                    .is_none()
                {
                    break 'body;
                }
            }

            // There is no implicit return of the final value of a procedure. That value is lost,
            // and unit is returned instead. It is most likely that there is a return in the body,
            // and this final return will be dead code.
            let _value = self.compile_expression(&mut scope, &procedure.body);
            if !self
                .builder
                .get_insert_block()
                .unwrap()
                .get_last_instruction()
                .unwrap()
                .is_terminator()
            {
                self.builder
                    .build_store(scope.sret(), self.unit_const())
                    .unwrap();
                self.builder.build_unconditional_branch(cleanup).unwrap();
            }
        }

        self.builder.position_at_end(no_match);
        _ = self.internal_panic(self.embed_c_string(format!(
            "no argument match in call to proc {}::{}!(...)\n",
            self.location, definition.name,
        )));

        self.builder.position_at_end(cleanup);
        for pointer in scope.variables.into_values() {
            self.trilogy_value_destroy(pointer);
        }
        self.builder.build_return(None).unwrap();
        self.di.pop_debug_scope();
    }
}
