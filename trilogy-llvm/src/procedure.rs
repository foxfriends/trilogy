use crate::{scope::Scope, Codegen};
use inkwell::{
    attributes::{Attribute, AttributeLoc},
    module::Linkage,
};
use trilogy_ir::ir;

impl Codegen<'_> {
    pub(crate) fn compile_procedure(&self, definition: &ir::ProcedureDefinition, linkage: Linkage) {
        assert_eq!(definition.overloads.len(), 1);
        let procedure = &definition.overloads[0];
        let fn_type = self.procedure_type(procedure.parameters.len());
        let function =
            self.module
                .add_function(&definition.name.to_string(), fn_type, Some(linkage));
        function.add_attribute(
            AttributeLoc::Param(0),
            self.context.create_type_attribute(
                Attribute::get_named_enum_kind_id("sret"),
                self.value_type().into(),
            ),
        );

        let mut scope = Scope::begin(function);
        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);

        for (n, param) in procedure.parameters.iter().enumerate() {
            let value = function
                .get_nth_param(n as u32 + 1)
                .unwrap()
                .into_struct_value();
            self.compile_pattern_match(&mut scope, param, value);
        }

        // There is no implicit return of the final value of a procedure. That value is lost,
        // and unit is returned instead. It is most likely that there is a return in the body,
        // and this final return will be dead code.
        let _value = self.compile_expression(&mut scope, &procedure.body);
        self.builder.build_return(None).unwrap();
    }
}
