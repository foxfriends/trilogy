use crate::{Codegen, IMPLICIT_PARAMS};
use inkwell::module::Linkage;
use inkwell::values::{BasicMetadataValueEnum, FunctionValue, GlobalValue};
use source_span::Span;
use trilogy_ir::ir::CallConv;
use trilogy_ir::{Id, ir};

const MAIN_NAME: &str = "trilogy:::main";

impl<'ctx> Codegen<'ctx> {
    fn write_procedure_accessor(
        &self,
        definition: &ir::ProcedureDefinition,
        accessor: FunctionValue<'ctx>,
        accessing: FunctionValue<'ctx>,
    ) -> GlobalValue<'ctx> {
        let has_context = accessor.count_params() == 2;
        let accessor_entry = self.context.append_basic_block(accessor, "entry");
        self.builder.position_at_end(accessor_entry);
        let sret = accessor.get_nth_param(0).unwrap().into_pointer_value();
        let metadata = self.build_callable_data(
            &self.module_path(),
            &definition.name.to_string(),
            definition.arity as u32,
            definition.span(),
            None,
        );
        if has_context {
            let ctx = accessor.get_nth_param(1).unwrap().into_pointer_value();
            self.trilogy_callable_init_do(sret, definition.arity, ctx, accessing, metadata);
        } else {
            self.trilogy_callable_init_proc(sret, definition.arity, accessing, metadata);
        }
        self.builder.build_return(None).unwrap();
        metadata
    }

    pub(crate) fn declare_extern_procedure(
        &self,
        definition: &ir::ProcedureDefinition,
        linkage: Linkage,
        span: Span,
    ) -> FunctionValue<'ctx> {
        let name = definition.name.to_string();
        let accessor_name = format!("{}::{}", self.module_path(), name);
        match definition.call_conv {
            CallConv::C => {
                let wrapper_name = format!("{}::{}.tailcc", self.module_path(), name);

                let original_function =
                    self.add_external_declaration(&name, definition.arity, span);
                // To allow callers to always use FastCC, we provide a wrapper around all extern procedures that
                // converts to CCC.
                let wrapper_function =
                    self.add_procedure(&wrapper_name, definition.arity, &wrapper_name, span, true);

                // Build accessor first
                let accessor = self.add_accessor(&accessor_name, false, linkage);
                self.builder.unset_current_debug_location();
                let metadata =
                    self.write_procedure_accessor(definition, accessor, wrapper_function);

                self.set_current_definition(
                    wrapper_name.to_owned(),
                    wrapper_name.to_owned(),
                    span,
                    metadata,
                    None,
                );
                self.begin_function(wrapper_function, span);
                self.set_span(span);
                let ret_val = self.allocate_value("");
                let mut params = vec![ret_val.into()];
                params.extend(
                    self.function_params
                        .borrow()
                        .iter()
                        .skip(IMPLICIT_PARAMS)
                        .map(|val| BasicMetadataValueEnum::<'ctx>::from(*val)),
                );
                self.builder
                    .build_direct_call(original_function, &params, "")
                    .unwrap();
                self.call_known_continuation(self.get_return(""), ret_val);

                self.close_continuation();
                self.end_function();

                accessor
            }
            CallConv::Trilogy => {
                let function = match self.module.get_function(&name) {
                    Some(func) => func,
                    None => self.module.add_function(
                        &name,
                        self.procedure_type(definition.arity),
                        Some(Linkage::External),
                    ),
                };
                // Build accessor only, the function is already correct
                let accessor = self.add_accessor(&accessor_name, false, linkage);
                self.builder.unset_current_debug_location();
                self.write_procedure_accessor(definition, accessor, function);
                accessor
            }
        }
    }

    pub(crate) fn compile_procedure(
        &self,
        definition: &ir::ProcedureDefinition,
        module_context: Option<Vec<Id>>,
    ) {
        assert_eq!(definition.overloads.len(), 1);
        let procedure = &definition.overloads[0];
        let arity = procedure.parameters.len();
        let name = definition.name.to_string();
        let accessor_name = format!("{}::{}", self.module_path(), name);
        let accessor = self.module.get_function(&accessor_name).unwrap();

        let (function, linkage_name) = if name == "main" {
            assert_eq!(arity, 0);
            let function = self.add_main(MAIN_NAME, &name, definition.span());
            (function, MAIN_NAME)
        } else {
            let function = self.add_procedure(&name, arity, &name, definition.span(), false);
            (function, name.as_str())
        };
        let metadata = self.write_procedure_accessor(definition, accessor, function);
        self.set_current_definition(
            name.to_owned(),
            linkage_name.to_owned(),
            procedure.span,
            metadata,
            module_context,
        );
        self.compile_procedure_body(function, procedure);
        self.close_continuation();
    }

    pub(crate) fn compile_procedure_body(
        &self,
        function: FunctionValue<'ctx>,
        procedure: &ir::Procedure,
    ) {
        self.begin_function(function, procedure.span);
        'body: {
            self.set_span(procedure.head_span);
            for (n, param) in procedure.parameters.iter().enumerate() {
                let value = self.function_params.borrow()[n + IMPLICIT_PARAMS];
                let end = self.get_end_temporary();
                self.bind_temporary(end);
                if self.compile_pattern_match(param, value, end).is_none() {
                    break 'body;
                }
            }

            if let Some(value) = self.compile_expression(&procedure.body, "") {
                let ret = self.get_return("");
                self.call_known_continuation(ret, value);
            }
        }
        self.end_function();
    }
}
