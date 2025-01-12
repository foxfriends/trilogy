use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{scope::Scope, types};
use inkwell::{
    attributes::{Attribute, AttributeLoc},
    builder::Builder,
    context::Context,
    execution_engine::ExecutionEngine,
    module::{Linkage, Module},
    values::{FunctionValue, PointerValue},
    IntPredicate, OptimizationLevel,
};
use trilogy_ir::{ir, Id};

pub(crate) enum Head {
    Constant,
    Function,
    Procedure(usize),
    Rule(usize),
    Module(String),
}

pub(crate) struct Codegen<'ctx> {
    pub(crate) atoms: Rc<RefCell<HashMap<String, u64>>>,
    pub(crate) context: &'ctx Context,
    pub(crate) module: Module<'ctx>,
    pub(crate) builder: Builder<'ctx>,
    pub(crate) execution_engine: ExecutionEngine<'ctx>,
    pub(crate) modules: &'ctx HashMap<String, Option<&'ctx ir::Module>>,
    pub(crate) globals: HashMap<Id, Head>,
    pub(crate) location: String,
}

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn new(
        context: &'ctx Context,
        modules: &'ctx HashMap<String, Option<&'ctx ir::Module>>,
    ) -> Self {
        let module = context.create_module("trilogy:runtime");
        let mut atoms = HashMap::new();
        atoms.insert("undefined".to_owned(), types::TAG_UNDEFINED);
        atoms.insert("unit".to_owned(), types::TAG_UNIT);
        atoms.insert("bool".to_owned(), types::TAG_BOOL);
        atoms.insert("atom".to_owned(), types::TAG_ATOM);
        atoms.insert("char".to_owned(), types::TAG_CHAR);
        atoms.insert("string".to_owned(), types::TAG_STRING);
        atoms.insert("integer".to_owned(), types::TAG_INTEGER);
        atoms.insert("bits".to_owned(), types::TAG_BITS);
        atoms.insert("struct".to_owned(), types::TAG_STRUCT);
        atoms.insert("tuple".to_owned(), types::TAG_TUPLE);
        atoms.insert("array".to_owned(), types::TAG_ARRAY);
        atoms.insert("set".to_owned(), types::TAG_SET);
        atoms.insert("record".to_owned(), types::TAG_RECORD);
        atoms.insert("callable".to_owned(), types::TAG_CALLABLE);
        let codegen = Codegen {
            atoms: Rc::new(RefCell::new(atoms)),
            builder: context.create_builder(),
            context,
            execution_engine: module
                .create_jit_execution_engine(OptimizationLevel::None)
                .unwrap(),
            module,
            modules,
            globals: HashMap::default(),
            location: "trilogy:runtime".to_owned(),
        };

        let submodule = codegen.sub("trilogy:c");
        submodule.std_libc();

        codegen.module.link_in_module(submodule.module).unwrap();
        codegen
    }

    pub(crate) fn compile_entrypoint(&self, entrymodule: &str, entrypoint: &str) {
        let main_wrapper =
            self.module
                .add_function("main", self.context.i32_type().fn_type(&[], false), None);
        let scope = Scope::begin(main_wrapper);
        let basic_block = self.context.append_basic_block(main_wrapper, "entry");
        let exit_unit = self.context.append_basic_block(main_wrapper, "exit_unit");
        let exit_int = self.context.append_basic_block(main_wrapper, "exit_int");

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
        let tag = self.get_tag(output);
        let is_unit = self
            .builder
            .build_int_compare(
                IntPredicate::EQ,
                tag,
                self.tag_type().const_int(types::TAG_UNIT, false),
                "",
            )
            .unwrap();
        self.builder
            .build_conditional_branch(is_unit, exit_unit, exit_int)
            .unwrap();

        self.builder.position_at_end(exit_unit);
        self.builder
            .build_return(Some(&self.context.i32_type().const_int(0, false)))
            .unwrap();

        self.builder.position_at_end(exit_int);
        let exit_code = self.untag_integer(&scope, output);
        let exit_code = self
            .builder
            .build_int_truncate(exit_code, self.context.i32_type(), "")
            .unwrap();
        self.builder.build_return(Some(&exit_code)).unwrap();
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
