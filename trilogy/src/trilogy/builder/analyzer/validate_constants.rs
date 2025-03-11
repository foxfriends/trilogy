#![allow(dead_code)]

use super::prelude::*;
use daggy::{Dag, NodeIndex};
use std::collections::{HashMap, HashSet};
use trilogy_ir::Id;
use trilogy_ir::ir::{self, ModuleCell};
use trilogy_ir::visitor::{HasBindings, IrVisitable, IrVisitor};

#[derive(Copy, Clone)]
struct Node {
    index: NodeIndex<usize>,
    arity: Option<usize>,
}

pub(super) fn validate_constants<E: std::error::Error>(
    modules: &mut Modules,
    report: &mut ReportBuilder<E>,
) {
    // Compute a DAG over every constant and module in the program.
    // If a cycle is detected, it is considered an error.
    //
    // At the same time, we'll be looking for any applications that
    // are not statically determined to be modules, as functions,
    // procedures, and rules are all not allowed at constant time.

    // First, collect all the nodes so that the DAG can be initialized
    // with all of them up front.
    let mut nodes = HashMap::new();
    let mut dag = Dag::<usize, usize, usize>::new();
    // Also save an index of imported locations and exported names to
    // make resolving those easy.
    let mut exports = Exports::new();
    let mut imports = Imports::new();
    for (location, module) in modules.iter() {
        for def in module.definitions() {
            use ir::DefinitionItem::*;
            if def.is_exported {
                exports.insert(
                    (
                        location.clone(),
                        def.name().unwrap().name().to_owned(),
                    ),
                    def.name().unwrap().clone(),
                );
            }
            match &def.item {
                Procedure(def) => {
                    let index = dag.add_node(0);
                    nodes.insert(
                        def.name.id.clone(),
                        Node {
                            index,
                            arity: Some(0),
                        },
                    );
                }
                Function(def) => {
                    let index = dag.add_node(0);
                    nodes.insert(
                        def.name.id.clone(),
                        Node {
                            index,
                            arity: Some(0),
                        },
                    );
                }
                Rule(def) => {
                    let index = dag.add_node(0);
                    nodes.insert(
                        def.name.id.clone(),
                        Node {
                            index,
                            arity: Some(0),
                        },
                    );
                }
                Constant(def) => {
                    let index = dag.add_node(0);
                    nodes.insert(def.name.id.clone(), Node { index, arity: None });
                }
                Module(def) => {
                    let arity = match def.module.as_ref() {
                        ModuleCell::External(loc) => {
                            let location = location.relative(loc);
                            imports.insert(def.name.id.clone(), location);
                            0
                        }
                        ModuleCell::Module(def) => {
                            let parameters = &def.get().unwrap().parameters;
                            for parameter in parameters {
                                for id in parameter.bindings() {
                                    let index = dag.add_node(0);
                                    nodes.insert(id.clone(), Node { index, arity: None });
                                }
                            }
                            parameters.len()
                        }
                    };
                    let index = dag.add_node(0);
                    nodes.insert(
                        def.name.id.clone(),
                        Node {
                            index,
                            arity: Some(arity),
                        },
                    );
                }
                Test(..) => {}
            }
        }
    }

    // Now traverse every node again to determine the edges
    for (location, module) in modules.iter() {
        for def in module.definitions() {
            use ir::DefinitionItem::*;
            match &def.item {
                Constant(def) => {
                    let node = nodes.get(&def.name.id).unwrap();
                    for id in def.static_references(&exports, &imports) {
                        if dag
                            .update_edge(node.index, nodes.get(&id).unwrap().index, 0)
                            .is_err()
                        {
                            // TODO: see if we can use the error's contents to provide a better error message?
                            //       it should be possible/not that hard to determine what the cycle is.
                            report.error(Error::analysis(
                                location.clone(),
                                ErrorKind::ConstantCycle { def: *def.clone() },
                            ));
                        }
                    }
                }
                // TODO: determining module's references is more tricky, so we just best guess it here.
                // * For 0 arity modules, it's not that bad (all the references of its internals)
                // * For >0 arity modules, it depends on the values being passed in - some of
                //   those might be modules too, so the dependencies can only be determined at
                //   the time of constant evaluation. This is the part we don't check properly, and
                //   will end up with some false negatives, but it's rare and unimportant right now.
                Module(def) => {
                    let Some(module) = def.module.as_module() else {
                        continue;
                    };
                    let node = nodes.get(&def.name.id).unwrap();
                    for id in module.static_references(&exports, &imports) {
                        if dag
                            .update_edge(node.index, nodes.get(&id).unwrap().index, 0)
                            .is_err()
                        {
                            // TODO: see if we can use the error's contents to provide a better error message?
                            //       it should be possible/not that hard to determine what the cycle is.
                            report.error(Error::analysis(
                                location.clone(),
                                ErrorKind::ModuleCycle { def: *def.clone() },
                            ));
                        }
                    }
                }

                Procedure(..) | Function(..) | Rule(..) | Test(..) => {}
            }
        }
    }
}

type Exports = HashMap<(Location, String), Id>;
type Imports = HashMap<Id, Location>;

struct StaticReferences<'a> {
    exports: &'a Exports,
    imports: &'a Imports,
    references: HashSet<Id>,
}

impl IrVisitor for StaticReferences<'_> {
    fn visit_constant_definition(&mut self, node: &ir::ConstantDefinition) {
        node.value.visit(self);
    }

    fn visit_reference(&mut self, node: &ir::Identifier) {
        self.references.insert(node.id.clone());
    }

    fn visit_definition(&mut self, node: &ir::Definition) {
        use ir::DefinitionItem::*;
        match &node.item {
            Constant(val) => val.visit(self),
            Module(val) => val.visit(self),
            _ => {}
        }
    }
}

trait HasStaticReferences: IrVisitable {
    fn static_references(&self, exports: &Exports, imports: &Imports) -> HashSet<Id> {
        let mut sr = StaticReferences {
            exports,
            imports,
            references: HashSet::default(),
        };
        self.visit(&mut sr);
        sr.references
    }
}

impl HasStaticReferences for ir::Module {}
impl HasStaticReferences for ir::ConstantDefinition {}
