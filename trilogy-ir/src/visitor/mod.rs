use crate::ir::*;

mod bindings;
mod can_evaluate;
mod identifiers;
mod references;

pub use bindings::{Bindings, HasBindings};
pub use can_evaluate::{CanEvaluate, HasCanEvaluate};
pub use identifiers::Identifiers;
pub use references::{HasReferences, References};

macro_rules! visit_node {
    ($name:ident, $t:ty) => {
        fn $name(&mut self, node: &$t) {
            node.visit(self);
        }
    };
}

macro_rules! visit_via {
    ($name:ident, $t:ty, $via:ident) => {
        fn $name(&mut self, node: &$t) {
            self.$via(node);
        }
    };
}

macro_rules! visit_many {
    ($name:ident, $t:ty) => {
        fn $name(&mut self, nodes: &[$t]) {
            for node in nodes {
                node.visit(self);
            }
        }
    };
}

macro_rules! visit_pair {
    ($name:ident, $t:ty) => {
        fn $name(&mut self, node: &($t, $t)) {
            node.0.visit(self);
            node.1.visit(self);
        }
    };
}

macro_rules! visit_value {
    ($name:ident, $t:ty) => {
        fn $name(&mut self, _value: &$t) {}
    };
}

macro_rules! visit_unit {
    ($name:ident) => {
        fn $name(&mut self) {}
    };
}

pub trait IrVisitor: Sized {
    visit_node!(visit_definition, Definition);
    visit_node!(visit_test_definition, TestDefinition);
    visit_node!(visit_procedure_definition, ProcedureDefinition);
    visit_node!(visit_procedure, Procedure);
    visit_node!(visit_function_definition, FunctionDefinition);
    visit_node!(visit_function, Function);
    visit_node!(visit_module_definition, ModuleDefinition);
    visit_node!(visit_module, Module);
    visit_node!(visit_rule_definition, RuleDefinition);
    visit_node!(visit_rule, Rule);
    visit_node!(visit_constant_definition, ConstantDefinition);
    visit_node!(visit_expression, Expression);
    visit_node!(visit_pattern, Expression);
    visit_node!(visit_value, Value);
    visit_value!(visit_builtin, Builtin);
    visit_node!(visit_pack, Pack);
    visit_node!(visit_element, Element);
    visit_many!(visit_sequence, Expression);
    visit_node!(visit_assignment, Assignment);
    visit_pair!(visit_mapping, Expression);
    visit_value!(visit_number, Number);
    visit_value!(visit_character, char);
    visit_value!(visit_string, str);
    visit_value!(visit_bits, Bits);
    visit_value!(visit_boolean, bool);
    visit_unit!(visit_unit);
    visit_pair!(visit_conjunction, Expression);
    visit_pair!(visit_disjunction, Expression);
    visit_unit!(visit_wildcard);
    visit_value!(visit_atom, str);
    visit_node!(visit_query, Query);
    visit_node!(visit_iterator, Iterator);
    visit_node!(visit_while, While);
    visit_node!(visit_for, Iterator);
    visit_node!(visit_application, Application);
    visit_node!(visit_let, Let);
    visit_node!(visit_if_else, IfElse);
    visit_node!(visit_match, Match);
    visit_node!(visit_case, Case);
    visit_via!(visit_fn, Function, visit_function);
    visit_via!(visit_do, Procedure, visit_procedure);
    visit_via!(visit_qy, Rule, visit_rule);
    visit_node!(visit_handled, Handled);
    visit_via!(visit_module_reference, Identifier, visit_identifier);
    visit_via!(visit_reference, Identifier, visit_identifier);
    visit_value!(visit_identifier, Identifier);
    visit_value!(visit_dynamic, trilogy_parser::syntax::Identifier);
    visit_node!(visit_assert, Assert);
    visit_unit!(visit_end);
    visit_node!(visit_query_value, QueryValue);
    visit_pair!(visit_query_disjunction, Query);
    visit_pair!(visit_query_conjunction, Query);
    visit_pair!(visit_query_implication, Query);
    visit_pair!(visit_query_alternative, Query);
    visit_node!(visit_direct_unification, Unification);
    visit_node!(visit_element_unification, Unification);
    visit_node!(visit_lookup, Lookup);
    visit_node!(visit_query_is, Expression);
    visit_node!(visit_query_not, Query);
    visit_unit!(visit_query_pass);
    visit_unit!(visit_query_fail);
    visit_node!(visit_handler, Handler);
}

pub trait IrVisitable {
    fn visit<V: IrVisitor>(&self, visitor: &mut V);
}

impl IrVisitable for Expression {
    fn visit<V: IrVisitor>(&self, visitor: &mut V) {
        visitor.visit_value(&self.value);
    }
}

impl IrVisitable for Value {
    fn visit<V: IrVisitor>(&self, visitor: &mut V) {
        use Value::*;

        match self {
            Builtin(val) => visitor.visit_builtin(val),
            Pack(val) => visitor.visit_pack(val),
            Sequence(val) => visitor.visit_sequence(val),
            Assignment(val) => visitor.visit_assignment(val),
            Mapping(val) => visitor.visit_mapping(val),
            Number(val) => visitor.visit_number(val),
            Character(val) => visitor.visit_character(val),
            String(val) => visitor.visit_string(val),
            Bits(val) => visitor.visit_bits(val),
            Boolean(val) => visitor.visit_boolean(val),
            Unit => visitor.visit_unit(),
            Conjunction(val) => visitor.visit_conjunction(val),
            Disjunction(val) => visitor.visit_disjunction(val),
            Wildcard => visitor.visit_wildcard(),
            Atom(val) => visitor.visit_atom(val),
            Query(val) => visitor.visit_query(val),
            Iterator(val) => visitor.visit_iterator(val),
            While(val) => visitor.visit_while(val),
            For(val) => visitor.visit_for(val),
            Application(val) => visitor.visit_application(val),
            Let(val) => visitor.visit_let(val),
            IfElse(val) => visitor.visit_if_else(val),
            Match(val) => visitor.visit_match(val),
            Fn(val) => visitor.visit_fn(val),
            Do(val) => visitor.visit_do(val),
            Qy(val) => visitor.visit_qy(val),
            Handled(val) => visitor.visit_handled(val),
            Reference(val) => visitor.visit_reference(val),
            Dynamic(val) => visitor.visit_dynamic(val),
            Assert(val) => visitor.visit_assert(val),
            End => visitor.visit_end(),
        }
    }
}

impl IrVisitable for Pack {
    fn visit<V: IrVisitor>(&self, visitor: &mut V) {
        for value in &self.values {
            visitor.visit_element(value);
        }
    }
}

impl IrVisitable for Element {
    fn visit<V: IrVisitor>(&self, visitor: &mut V) {
        visitor.visit_expression(&self.expression);
    }
}

impl IrVisitable for Assignment {
    fn visit<V: IrVisitor>(&self, visitor: &mut V) {
        visitor.visit_expression(&self.lhs);
        visitor.visit_expression(&self.rhs);
    }
}

impl IrVisitable for Query {
    fn visit<V: IrVisitor>(&self, visitor: &mut V) {
        visitor.visit_query_value(&self.value);
    }
}

impl IrVisitable for QueryValue {
    fn visit<V: IrVisitor>(&self, visitor: &mut V) {
        use QueryValue::*;

        match self {
            Disjunction(val) => visitor.visit_query_disjunction(val),
            Conjunction(val) => visitor.visit_query_conjunction(val),
            Implication(val) => visitor.visit_query_implication(val),
            Alternative(val) => visitor.visit_query_alternative(val),
            Direct(val) => visitor.visit_direct_unification(val),
            Element(val) => visitor.visit_element_unification(val),
            Lookup(val) => visitor.visit_lookup(val),
            Is(val) => visitor.visit_query_is(val),
            Not(val) => visitor.visit_query_not(val),
            Pass => visitor.visit_query_pass(),
            End => visitor.visit_query_fail(),
        }
    }
}

impl IrVisitable for Iterator {
    fn visit<V: IrVisitor>(&self, visitor: &mut V) {
        visitor.visit_expression(&self.value);
        visitor.visit_query(&self.query);
    }
}

impl IrVisitable for While {
    fn visit<V: IrVisitor>(&self, visitor: &mut V) {
        visitor.visit_expression(&self.condition);
        visitor.visit_expression(&self.body);
    }
}

impl IrVisitable for Application {
    fn visit<V: IrVisitor>(&self, visitor: &mut V) {
        visitor.visit_expression(&self.function);
        visitor.visit_expression(&self.argument);
    }
}

impl IrVisitable for Let {
    fn visit<V: IrVisitor>(&self, visitor: &mut V) {
        visitor.visit_query(&self.query);
        visitor.visit_expression(&self.body);
    }
}

impl IrVisitable for IfElse {
    fn visit<V: IrVisitor>(&self, visitor: &mut V) {
        visitor.visit_expression(&self.condition);
        visitor.visit_expression(&self.when_true);
        visitor.visit_expression(&self.when_false);
    }
}

impl IrVisitable for Match {
    fn visit<V: IrVisitor>(&self, visitor: &mut V) {
        visitor.visit_expression(&self.expression);
        for case in &self.cases {
            visitor.visit_case(case);
        }
    }
}

impl IrVisitable for Case {
    fn visit<V: IrVisitor>(&self, visitor: &mut V) {
        visitor.visit_pattern(&self.pattern);
        visitor.visit_expression(&self.guard);
        visitor.visit_expression(&self.body);
    }
}

impl IrVisitable for Function {
    fn visit<V: IrVisitor>(&self, visitor: &mut V) {
        for parameter in &self.parameters {
            visitor.visit_pattern(parameter);
        }
        visitor.visit_expression(&self.body);
    }
}

impl IrVisitable for Procedure {
    fn visit<V: IrVisitor>(&self, visitor: &mut V) {
        for parameter in &self.parameters {
            visitor.visit_pattern(parameter);
        }
        visitor.visit_expression(&self.body);
    }
}

impl IrVisitable for Handled {
    fn visit<V: IrVisitor>(&self, visitor: &mut V) {
        visitor.visit_expression(&self.expression);
        for handler in &self.handlers {
            visitor.visit_handler(handler);
        }
    }
}

impl IrVisitable for Handler {
    fn visit<V: IrVisitor>(&self, visitor: &mut V) {
        visitor.visit_pattern(&self.pattern);
        visitor.visit_expression(&self.guard);
        visitor.visit_expression(&self.body);
    }
}

impl IrVisitable for Assert {
    fn visit<V: IrVisitor>(&self, visitor: &mut V) {
        visitor.visit_expression(&self.message);
        visitor.visit_expression(&self.assertion);
    }
}

impl IrVisitable for Unification {
    fn visit<V: IrVisitor>(&self, visitor: &mut V) {
        visitor.visit_pattern(&self.pattern);
        visitor.visit_expression(&self.expression);
    }
}

impl IrVisitable for Lookup {
    fn visit<V: IrVisitor>(&self, visitor: &mut V) {
        visitor.visit_expression(&self.path);
        for pattern in &self.patterns {
            visitor.visit_expression(pattern);
        }
    }
}

impl IrVisitable for RuleDefinition {
    fn visit<V: IrVisitor>(&self, visitor: &mut V) {
        visitor.visit_identifier(&self.name);
        for overload in &self.overloads {
            visitor.visit_rule(overload);
        }
    }
}

impl IrVisitable for ProcedureDefinition {
    fn visit<V: IrVisitor>(&self, visitor: &mut V) {
        visitor.visit_identifier(&self.name);
        for overload in &self.overloads {
            visitor.visit_procedure(overload);
        }
    }
}

impl IrVisitable for FunctionDefinition {
    fn visit<V: IrVisitor>(&self, visitor: &mut V) {
        visitor.visit_identifier(&self.name);
        for overload in &self.overloads {
            visitor.visit_function(overload);
        }
    }
}

impl IrVisitable for Rule {
    fn visit<V: IrVisitor>(&self, visitor: &mut V) {
        for parameter in &self.parameters {
            visitor.visit_pattern(parameter);
        }
        visitor.visit_query(&self.body);
    }
}

impl IrVisitable for ModuleDefinition {
    fn visit<V: IrVisitor>(&self, visitor: &mut V) {
        visitor.visit_identifier(&self.name);
        visitor.visit_module(self.module.as_module().unwrap());
    }
}

impl IrVisitable for Module {
    fn visit<V: IrVisitor>(&self, visitor: &mut V) {
        for parameter in &self.parameters {
            visitor.visit_pattern(parameter);
        }
        for definition in self.definitions() {
            visitor.visit_definition(definition);
        }
    }
}

impl IrVisitable for Definition {
    fn visit<V: IrVisitor>(&self, visitor: &mut V) {
        use DefinitionItem::*;

        match &self.item {
            Constant(val) => visitor.visit_constant_definition(val),
            Procedure(val) => visitor.visit_procedure_definition(val),
            Function(val) => visitor.visit_function_definition(val),
            Rule(val) => visitor.visit_rule_definition(val),
            Test(val) => visitor.visit_test_definition(val),
            Module(val) => visitor.visit_module_definition(val),
        }
    }
}

impl IrVisitable for TestDefinition {
    fn visit<V: IrVisitor>(&self, visitor: &mut V) {
        visitor.visit_string(&self.name);
        visitor.visit_expression(&self.body);
    }
}

impl IrVisitable for ConstantDefinition {
    fn visit<V: IrVisitor>(&self, visitor: &mut V) {
        visitor.visit_identifier(&self.name);
        visitor.visit_expression(&self.value);
    }
}
