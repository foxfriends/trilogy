use crate::{scope::Scope, Codegen};
use trilogy_ir::ir;

impl Codegen<'_> {
    pub(crate) fn import_procedure(&self, location: &str, definition: &ir::ProcedureDefinition) {
        assert_eq!(definition.overloads.len(), 1);
        let procedure = &definition.overloads[0];
        self.add_procedure(
            &format!("{}::{}", location, definition.name),
            procedure.parameters.len(),
            true,
        );
    }

    pub(crate) fn declare_procedure(&self, definition: &ir::ProcedureDefinition, exported: bool) {
        assert_eq!(definition.overloads.len(), 1);
        let procedure = &definition.overloads[0];
        self.add_procedure(
            &format!("{}::{}", self.location, definition.name),
            procedure.parameters.len(),
            exported,
        );
    }

    pub(crate) fn compile_procedure(&self, definition: &ir::ProcedureDefinition) {
        assert_eq!(definition.overloads.len(), 1);
        let procedure = &definition.overloads[0];
        let function = self
            .module
            .get_function(&format!("{}::{}", self.location, definition.name))
            .unwrap();

        let mut scope = Scope::begin(function);
        let basic_block = self.context.append_basic_block(function, "entry");
        let no_match = self.context.append_basic_block(function, "no_match");
        self.builder.position_at_end(basic_block);

        for (n, param) in procedure.parameters.iter().enumerate() {
            let value = function
                .get_nth_param(n as u32 + 1)
                .unwrap()
                .into_pointer_value();
            self.compile_pattern_match(&mut scope, param, value, no_match);
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
            self.builder.build_return(None).unwrap();
        }

        self.builder.position_at_end(no_match);
        self.panic("procedure call no match");
    }
}
