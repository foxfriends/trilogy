use crate::scope::Scope;
use inkwell::{
    attributes::{Attribute, AttributeLoc},
    builder::Builder,
    context::Context,
    execution_engine::ExecutionEngine,
    module::{Linkage, Module},
    values::{FunctionValue, PointerValue},
    OptimizationLevel,
};
use trilogy_ir::Id;

pub(crate) struct Codegen<'ctx> {
    pub(crate) context: &'ctx Context,
    pub(crate) module: Module<'ctx>,
    pub(crate) builder: Builder<'ctx>,
    pub(crate) execution_engine: ExecutionEngine<'ctx>,
}

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn new(context: &'ctx Context) -> Self {
        let module = context.create_module("trilogy:runtime");
        let codegen = Codegen {
            builder: context.create_builder(),
            context,
            execution_engine: module
                .create_jit_execution_engine(OptimizationLevel::None)
                .unwrap(),
            module,
        };

        let submodule = codegen.sub("trilogy:c");
        submodule.std_libc();

        codegen.module.link_in_module(submodule.module).unwrap();
        codegen
    }

    pub(crate) fn compile_entrypoint(&self, entrymodule: &str, entrypoint: &str) {
        let main_wrapper =
            self.module
                .add_function("main", self.context.i8_type().fn_type(&[], false), None);
        let basic_block = self.context.append_basic_block(main_wrapper, "entry");
        self.builder.position_at_end(basic_block);
        let main = self
            .module
            .get_function(&format!("{entrymodule}::{entrypoint}"))
            .unwrap();
        let output = self
            .builder
            .build_alloca(self.value_type(), "output")
            .unwrap();
        self.builder
            .build_direct_call(main, &[output.into()], "main")
            .unwrap();
        let exitcode = self
            .builder
            .build_struct_gep(self.value_type(), output, 0, "exitcode")
            .unwrap();
        let exitcode = self
            .builder
            .build_load(self.tag_type(), exitcode, "exitcode")
            .unwrap();
        self.builder.build_return(Some(&exitcode)).unwrap();
    }

    pub(crate) fn add_procedure(
        &self,
        name: &str,
        arity: usize,
        exported: bool,
    ) -> FunctionValue<'ctx> {
        let procedure = self.module.add_function(
            name,
            self.procedure_type(arity),
            if exported {
                Some(Linkage::External)
            } else {
                Some(Linkage::Private)
            },
        );
        procedure.add_attribute(
            AttributeLoc::Param(0),
            self.context.create_type_attribute(
                Attribute::get_named_enum_kind_id("sret"),
                self.value_type().into(),
            ),
        );
        procedure.get_nth_param(0).unwrap().set_name("sretptr");
        procedure
    }

    pub(crate) fn variable(&self, scope: &mut Scope<'ctx>, id: Id) -> PointerValue<'ctx> {
        if scope.variables.contains_key(&id) {
            return *scope.variables.get(&id).unwrap();
        }

        let builder = self.context.create_builder();
        let entry = scope.function.get_first_basic_block().unwrap();
        match entry.get_first_instruction() {
            Some(instruction) => builder.position_before(&instruction),
            None => builder.position_at_end(entry),
        }
        let variable = builder
            .build_alloca(self.value_type(), &id.to_string())
            .unwrap();
        scope.variables.insert(id, variable);
        variable
    }
}
