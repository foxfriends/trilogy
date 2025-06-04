use crate::Codegen;
use inkwell::module::Linkage;
use inkwell::values::{BasicMetadataValueEnum, FunctionValue};
use source_span::Span;
use trilogy_ir::ir;

const MAIN_NAME: &str = "trilogy:::main";
// NOTE: params start at 7, due to return, yield, end, cancel, resume, break, and continue
const PROCEDURE_IMPLICIT_PARAMS: usize = 7;

impl<'ctx> Codegen<'ctx> {
    fn write_procedure_accessor(
        &self,
        accessor: FunctionValue<'ctx>,
        accessing: FunctionValue<'ctx>,
        arity: usize,
    ) {
        let has_context = accessor.count_params() == 2;
        assert!(!has_context, "TODO");
        let sret = accessor.get_nth_param(0).unwrap().into_pointer_value();
        let accessor_entry = self.context.append_basic_block(accessor, "entry");
        self.builder.position_at_end(accessor_entry);
        self.trilogy_callable_init_proc(
            sret,
            arity,
            accessing.as_global_value().as_pointer_value(),
        );
        self.builder.build_return(None).unwrap();
    }

    pub(crate) fn declare_extern_procedure(
        &self,
        name: &str,
        arity: usize,
        linkage: Linkage,
        span: Span,
    ) -> FunctionValue<'ctx> {
        let accessor_name = format!("{}::{}", self.module_path(), name);
        let wrapper_name = format!("{}::{}.fastcc", self.module_path(), name);
        let original_function = self.add_external_declaration(name, arity, span);

        // To allow callers to always use FastCC, we provide a wrapper around all extern procedures that
        // converts to CCC.
        let wrapper_function = self.add_procedure(&wrapper_name, arity, &wrapper_name, span, true);
        self.begin_function(wrapper_function, span);
        self.set_span(span);
        self.set_current_definition(wrapper_name.to_owned(), wrapper_name.to_owned(), span);
        let ret_val = self.allocate_value("");
        let mut params = vec![ret_val.into()];
        params.extend(
            self.function_params
                .borrow()
                .iter()
                .skip(PROCEDURE_IMPLICIT_PARAMS)
                .map(|val| BasicMetadataValueEnum::<'ctx>::from(*val)),
        );
        self.builder
            .build_direct_call(original_function, &params, "")
            .unwrap();
        self.call_known_continuation(self.get_return(""), ret_val);

        self.close_continuation();
        self.end_function();

        let accessor = self.add_accessor(&accessor_name, false, linkage);
        self.set_current_definition(name.to_owned(), accessor_name.to_owned(), span);
        self.builder.unset_current_debug_location();
        self.write_procedure_accessor(accessor, wrapper_function, arity);

        accessor
    }

    pub(crate) fn compile_procedure(&self, definition: &ir::ProcedureDefinition) {
        assert_eq!(definition.overloads.len(), 1);
        let procedure = &definition.overloads[0];
        let arity = procedure.parameters.len();
        let name = definition.name.to_string();
        let linkage_name = if name == "main" { MAIN_NAME } else { &name };
        let accessor_name = format!("{}::{}", self.module_path(), name);
        let accessor = self.module.get_function(&accessor_name).unwrap();
        let function = self.add_procedure(linkage_name, arity, &name, definition.span(), false);
        self.write_procedure_accessor(accessor, function, arity);
        self.set_current_definition(name.to_owned(), linkage_name.to_owned(), procedure.span);
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
                let value = self.function_params.borrow()[n + PROCEDURE_IMPLICIT_PARAMS];
                if self
                    .compile_pattern_match(param, value, self.get_end(""))
                    .is_none()
                {
                    break 'body;
                }
            }

            if let Some(value) = self.compile_expression(&procedure.body, "") {
                // There is no implicit return of the final value of a procedure. That value is lost,
                // and unit is returned instead. It is most likely that there is a return in the body,
                // in which case we never reach this point
                self.trilogy_value_destroy(value);
                let ret = self.get_return("");
                self.call_known_continuation(ret, self.allocate_const(self.unit_const(), ""));
            }
        }
        self.end_function();
    }
}
