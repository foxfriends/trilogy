use inkwell::{
    builder::Builder,
    context::Context,
    module::{Linkage, Module},
    types::StructType,
};
use trilogy_ir::ir::{self, DefinitionItem};

pub(crate) struct Codegen<'ctx> {
    pub(crate) context: &'ctx Context,
    pub(crate) module: Module<'ctx>,
    pub(crate) builder: Builder<'ctx>,
}

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn new(context: &'ctx Context) -> Self {
        let codegen = Codegen {
            module: context.create_module("trilogy:runtime"),
            builder: context.create_builder(),
            context,
        };

        codegen
    }

    pub(crate) fn value_type(&self) -> StructType<'ctx> {
        self.context.struct_type(
            &[
                self.context.i8_type().into(),
                self.context.i8_type().vec_type(8).into(),
            ],
            false,
        )
    }

    fn sub(&self, name: &str) -> Codegen<'ctx> {
        Codegen {
            context: self.context,
            module: self.context.create_module(name),
            builder: self.context.create_builder(),
        }
    }

    pub(crate) fn compile_module(&self, file: &str, module: &ir::Module) {
        let subcontext = self.sub(&format!("file:{}", file));
        for definition in module.definitions() {
            let linkage = if definition.is_exported {
                Linkage::External
            } else {
                Linkage::Private
            };
            match &definition.item {
                DefinitionItem::Procedure(procedure) => {
                    subcontext.compile_procedure(procedure, linkage);
                }
                _ => todo!(),
            }
        }
        self.module.link_in_module(subcontext.module).unwrap();
    }
}
