use crate::{codegen::Codegen, scope::Scope, types};
use inkwell::{values::FunctionValue, IntPredicate};

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn core(&self) {
        self.import_libc();
        self.define_structural_eq();
    }

    pub(crate) fn import_core(&self) {
        self.structural_eq();
    }

    pub(crate) fn structural_eq(&self) -> FunctionValue<'ctx> {
        if let Some(func) = self.module.get_function("trilogy:core::structural_eq") {
            return func;
        }
        self.add_function("trilogy:core::structural_eq", true)
    }

    fn define_structural_eq(&self) {
        let function = self.add_procedure("trilogy:core::structural_eq", 2, true);
        let scope = Scope::begin(function);
        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);
        let lhs = function.get_nth_param(1).unwrap().into_pointer_value();
        let rhs = function.get_nth_param(2).unwrap().into_pointer_value();

        let lhs_tag = self.get_tag(lhs);
        let rhs_tag = self.get_tag(rhs);
        let is_same_type = self
            .builder
            .build_int_compare(IntPredicate::EQ, lhs_tag, rhs_tag, "is_same_type")
            .unwrap();

        let from_block = self.builder.get_insert_block().unwrap();
        let cmp_block = self.context.append_basic_block(scope.function, "eq_cmp");
        let cont_block = self.context.append_basic_block(scope.function, "eq_cont");
        self.builder
            .build_conditional_branch(is_same_type, cmp_block, cont_block)
            .unwrap();

        self.builder.position_at_end(cmp_block);
        let literal_block = self.context.append_basic_block(scope.function, "eq_lit");
        let string_block = self.context.append_basic_block(scope.function, "eq_str");
        self.builder
            .build_switch(
                lhs_tag,
                cont_block,
                &[
                    (
                        self.tag_type().const_int(types::TAG_UNIT, false),
                        literal_block,
                    ),
                    (
                        self.tag_type().const_int(types::TAG_BOOL, false),
                        literal_block,
                    ),
                    (
                        self.tag_type().const_int(types::TAG_CHAR, false),
                        literal_block,
                    ),
                    (
                        self.tag_type().const_int(types::TAG_ATOM, false),
                        literal_block,
                    ),
                    (
                        self.tag_type().const_int(types::TAG_INTEGER, false),
                        literal_block,
                    ),
                    (
                        self.tag_type().const_int(types::TAG_CALLABLE, false),
                        literal_block,
                    ),
                    (
                        self.tag_type().const_int(types::TAG_STRING, false),
                        string_block,
                    ),
                ],
            )
            .unwrap();

        self.builder.position_at_end(literal_block);
        let lhs_payload = self.get_payload(lhs);
        let rhs_payload = self.get_payload(rhs);
        let is_eq_literal = self
            .builder
            .build_int_compare(IntPredicate::EQ, lhs_payload, rhs_payload, "lit_eq")
            .unwrap();
        self.builder.build_unconditional_branch(cont_block).unwrap();

        self.builder.position_at_end(string_block);
        let strcmp = self.strcmp();
        let result = self.call_procedure(strcmp, &[lhs.into(), rhs.into()], "strcmp.result");
        let result = self.get_payload(result);
        let is_eq_string = self
            .builder
            .build_int_compare(
                IntPredicate::EQ,
                result,
                self.context.i64_type().const_int(0, false),
                "",
            )
            .unwrap();
        self.builder.build_unconditional_branch(cont_block).unwrap();

        self.builder.position_at_end(cont_block);
        let phi = self
            .builder
            .build_phi(self.context.bool_type(), "eq")
            .unwrap();
        phi.add_incoming(&[
            (&self.context.bool_type().const_int(0, false), cmp_block),
            (&self.context.bool_type().const_int(0, false), from_block),
            (&is_eq_literal, literal_block),
            (&is_eq_string, string_block),
        ]);
        let retval = self.bool_value(phi.as_basic_value().into_int_value());
        let retval = self
            .builder
            .build_load(self.value_type(), retval, "")
            .unwrap();
        self.builder.build_store(scope.sret(), retval).unwrap();
        self.builder.build_return(None).unwrap();
    }
}
