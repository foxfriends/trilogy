use crate::{Codegen, IMPLICIT_PARAMS};
use inkwell::module::Linkage;
use inkwell::values::{BasicMetadataValueEnum, FunctionValue};
use source_span::Span;
use trilogy_ir::{Id, ir};

const MAIN_NAME: &str = "trilogy:::main";

impl<'ctx> Codegen<'ctx> {
    fn write_procedure_accessor(
        &self,
        accessor: FunctionValue<'ctx>,
        accessing: FunctionValue<'ctx>,
        arity: usize,
    ) {
        let has_context = accessor.count_params() == 2;
        let accessor_entry = self.context.append_basic_block(accessor, "entry");
        self.builder.position_at_end(accessor_entry);
        let sret = accessor.get_nth_param(0).unwrap().into_pointer_value();
        if has_context {
            let ctx = accessor.get_nth_param(1).unwrap().into_pointer_value();
            self.trilogy_callable_init_do(sret, arity, ctx, accessing);
        } else {
            self.trilogy_callable_init_proc(sret, arity, accessing);
        }
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
        let wrapper_name = format!("{}::{}.tailcc", self.module_path(), name);
        let original_function = self.add_external_declaration(name, arity, span);

        // To allow callers to always use FastCC, we provide a wrapper around all extern procedures that
        // converts to CCC.
        let wrapper_function = self.add_procedure(&wrapper_name, arity, &wrapper_name, span, true);
        self.set_current_definition(wrapper_name.to_owned(), wrapper_name.to_owned(), span, None);
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

        let accessor = self.add_accessor(&accessor_name, false, linkage);
        self.set_current_definition(name.to_owned(), accessor_name.to_owned(), span, None);
        self.builder.unset_current_debug_location();
        self.write_procedure_accessor(accessor, wrapper_function, arity);

        accessor
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
            let function = self.add_main(
                MAIN_NAME,
                &name,
                definition.span(),
                module_context.is_some(),
            );
            (function, MAIN_NAME)
        } else {
            let function = self.add_procedure(&name, arity, &name, definition.span(), false);
            (function, name.as_str())
        };
        self.write_procedure_accessor(accessor, function, arity);
        self.set_current_definition(
            name.to_owned(),
            linkage_name.to_owned(),
            procedure.span,
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
                let end = self.get_end("");
                self.bind_temporary(end);
                if self.compile_pattern_match(param, value, end).is_none() {
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
