use super::{Labeler, Scope};
use crate::evaluation::CodegenEvaluate;
use crate::needs::BreakContinue;
use crate::pattern_match::CodegenPatternMatch;
use crate::prelude::*;
use crate::query::CodegenQuery;
use crate::{delegate_label_maker, delegate_stack_tracker};
use source_span::Span;
use trilogy_ir::ir::{self, Iterator};
use trilogy_ir::visitor::HasBindings;
use trilogy_ir::Id;
use trilogy_vm::{
    delegate_chunk_writer, Annotation, ChunkBuilder, ChunkWriter, Instruction, Location, Offset,
};

pub(crate) struct Context<'a> {
    labeler: &'a mut Labeler,
    pub scope: Scope<'a>,
    builder: &'a mut ChunkBuilder,
    location: &'a str,
}

impl<'a> Context<'a> {
    pub fn new(
        location: &'a str,
        builder: &'a mut ChunkBuilder,
        labeler: &'a mut Labeler,
        scope: Scope<'a>,
    ) -> Self {
        Self {
            labeler,
            scope,
            builder,
            location,
        }
    }

    pub fn location(&self) -> &str {
        self.location
    }
}

delegate_chunk_writer!(Context<'_>, builder);
delegate_stack_tracker!(Context<'_>, scope);
delegate_label_maker!(Context<'_>, labeler);

impl<'a> Context<'a> {
    pub fn declare_variables(&mut self, variables: impl IntoIterator<Item = Id>) -> usize {
        let mut n = 0;
        for id in variables {
            if self.scope.declare_variable(id.clone()) {
                let label = self.labeler.var(&id);
                self.label(label);
                self.instruction(Instruction::Variable);
                n += 1;
            }
        }
        n
    }

    pub fn undeclare_variables(
        &mut self,
        variables: impl IntoIterator<Item = Id>,
        pop: bool,
    ) -> &mut Self {
        for id in variables {
            if self.scope.undeclare_variable(&id) && pop {
                let label = self.labeler.unvar(&id);
                self.label(label);
                self.instruction(Instruction::Pop);
            }
        }
        self
    }

    pub fn iterator(
        &mut self,
        iterator: &Iterator,
        r#continue: Option<Offset>,
        r#break: Option<Offset>,
    ) -> &mut Self {
        self.declare_variables(iterator.query.bindings());
        self.prepare_query(&iterator.query)
            .repeat(|context, exit| {
                context.execute_query(&iterator.query, exit).intermediate(); // state
                if let Some(r#break) = r#break {
                    context.push_break(r#break);
                }
                if let Some(r#continue) = r#continue {
                    context.push_continue(r#continue);
                }

                match &iterator.value.value {
                    ir::Value::Mapping(mapping) => {
                        context.evaluate(&mapping.0).intermediate();
                        context
                            .evaluate(&mapping.1)
                            .end_intermediate()
                            .instruction(Instruction::Cons);
                    }
                    other => {
                        context.evaluate(other);
                    }
                }

                if r#continue.is_some() {
                    context.pop_continue();
                }
                if r#break.is_some() {
                    context.pop_break();
                }
                context
                    .atom("next")
                    .instruction(Instruction::Construct)
                    .r#yield()
                    .instruction(Instruction::Pop) // resume value discarded
                    .end_intermediate(); // state no longer intermediate
            })
            .instruction(Instruction::Pop)
            .undeclare_variables(iterator.query.bindings(), true)
    }

    pub fn evaluate<E: CodegenEvaluate>(&mut self, value: &E) -> &mut Self {
        value.evaluate(self);
        self
    }

    pub fn evaluate_annotated<E: CodegenEvaluate>(
        &mut self,
        value: &E,
        note: impl Into<String>,
        span: Span,
    ) -> &mut Self {
        let start = self.ip();
        value.evaluate(self);
        let end = self.ip();
        self.annotate(Annotation::source(
            start,
            end,
            note.into(),
            Location::new(self.location(), span),
        ));
        self
    }

    pub fn pattern_match<E: CodegenPatternMatch>(&mut self, value: &E, on_fail: &str) -> &mut Self {
        value.pattern_match(self, on_fail);
        self
    }

    pub fn prepare_query<E: CodegenQuery>(&mut self, value: &E) -> &mut Self {
        value.prepare_query(self);
        self
    }

    pub fn extend_query_state<E: CodegenQuery>(&mut self, value: &E) -> &mut Self {
        value.extend_query_state(self);
        self
    }

    pub fn execute_query<E: CodegenQuery>(&mut self, value: &E, on_fail: &str) -> &mut Self {
        value.execute_query(self, on_fail);
        self
    }
}

impl Context<'_> {
    pub fn sequence(&mut self, seq: &[ir::Expression]) -> &mut Self {
        let mut seq = seq.iter();
        let Some(mut expr) = seq.next() else {
            // An empty sequence must still have a value
            return self.instruction(Instruction::Unit);
        };
        loop {
            self.evaluate(&expr.value);
            let Some(next_expr) = seq.next() else {
                break self;
            };
            expr = next_expr;
            self.instruction(Instruction::Pop);
        }
    }

    pub fn comprehension<F: FnOnce(&mut Context), G: FnOnce(&mut Context)>(
        &mut self,
        append: F,
        init: G,
    ) -> &mut Self {
        self.iterate(
            |context, params| {
                context
                    .instruction(Instruction::LoadLocal(params.cancel))
                    .instruction(Instruction::Swap)
                    .instruction(Instruction::LoadLocal(params.resume))
                    .instruction(Instruction::Unit)
                    .call_function()
                    .instruction(Instruction::Clone)
                    .instruction(Instruction::Swap)
                    .pipe(append)
                    .become_function();
            },
            init,
        )
    }

    pub fn r#while(&mut self, condition: &ir::Value, body: &ir::Value) -> &mut Self {
        let needs = BreakContinue::check(body);
        self.r#loop(
            |_| {},
            |context, done| {
                context
                    .evaluate(condition)
                    .typecheck("boolean")
                    .cond_jump(done);
            },
            |context| {
                context.evaluate(body);
            },
            |_| {},
            needs,
        )
        // Evaluation requires that an extra value ends up on the stack.
        // While "evaluates" to unit
        .instruction(Instruction::Unit)
    }

    pub fn r#for(&mut self, query: &ir::Query, body: &ir::Value) -> &mut Self {
        let did_match = self.constant(false).intermediate();
        let needs = BreakContinue::check(body);
        self.r#loop(
            |context| {
                context.declare_variables(query.bindings());
                context.prepare_query(query);
            },
            |context, done| {
                context
                    .execute_query(query, done)
                    // Mark down that this loop did get a match
                    .constant(true)
                    .instruction(Instruction::SetLocal(did_match))
                    .intermediate(); // query state
            },
            |context| {
                context.evaluate(body).end_intermediate(); // query state
            },
            |context| {
                context
                    .instruction(Instruction::Pop)
                    .undeclare_variables(query.bindings(), true);
            },
            needs,
        )
        .end_intermediate() // did match (no longer intermediate)
    }
}
