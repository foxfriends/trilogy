use crate::context::Context;
use crate::prelude::*;
use trilogy_ir::ir;
use trilogy_ir::visitor::{HasBindings, HasCanEvaluate};
use trilogy_vm::Instruction;

pub(crate) fn write_rule(context: &mut Context, rule: &ir::Rule, on_fail: &str) {
    let next = context.atom("next");
    let done = context.atom("done");

    let setup = context.labeler.unique_hint("setup");
    let end = context.labeler.unique_hint("end");
    let call = context.labeler.unique_hint("call");

    context
        .instruction(Instruction::Copy)
        .instruction(Instruction::Const(().into()))
        .instruction(Instruction::ValNeq)
        .cond_jump(&setup)
        .label(call.clone())
        .instruction(Instruction::Copy)
        .instruction(Instruction::Call(0))
        // After calling this overload's iterator, flatten failures into the parent.
        // The parent expects this to work more like a query than an iterator for
        // failures.
        .instruction(Instruction::Copy)
        .instruction(Instruction::Const(done.clone().into()))
        .instruction(Instruction::ValEq)
        .cond_jump(&end)
        .instruction(Instruction::Pop)
        .instruction(Instruction::Pop)
        .jump(on_fail);

    context.label(setup).instruction(Instruction::Pop);
    // First check all the parameters, make sure they work. If they don't
    // match, we can fail without even constructing the state.
    let mut cleanup = vec![];
    for (i, parameter) in rule.parameters.iter().enumerate() {
        let skip = context.labeler.unique_hint("skip");
        cleanup.push(context.labeler.unique_hint("cleanup"));
        context.declare_variables(parameter.bindings());
        context
            .instruction(Instruction::LoadLocal(1))
            .instruction(Instruction::Const(i.into()))
            .instruction(Instruction::Contains)
            .cond_jump(&skip)
            .instruction(Instruction::LoadLocal(2 + i as u32));
        write_pattern_match(context, parameter, &cleanup[i]);
        context.label(skip);
    }

    // Happy path: we continue by writing the query state down, and then
    // encapsulating all that into a closure which will be the iterator for
    // this overload of the rule.
    write_query_state(context, &rule.body);
    context.close(&call);

    // The actual body of the rule involves running the query, then
    // returning the return value in 'next. We convert failure to
    // returning 'done, as in a regular iterator.
    let on_done = context.labeler.unique_hint("on_done");
    let actual_state = context.scope.intermediate();
    context.instruction(Instruction::LoadLocal(actual_state));
    write_query(context, &rule.body, &on_done, Some(1));
    context.instruction(Instruction::SetLocal(actual_state));
    context.scope.end_intermediate();
    // The query is normal, then the value is computed by evaluating
    // the parameter patterns now as expressions.
    context.scope.intermediate(); // At this point, the query state is an intermediate
    for (i, param) in rule.parameters.iter().enumerate() {
        let eval = context.labeler.unique_hint("eval");
        let next = context.labeler.unique_hint("next");
        context
            .instruction(Instruction::LoadLocal(1))
            .instruction(Instruction::Const(i.into()))
            .instruction(Instruction::Contains)
            .cond_jump(&eval)
            .instruction(Instruction::LoadLocal(2 + i as u32))
            .jump(&next);
        context.label(eval);
        if param.can_evaluate() {
            write_expression(context, param);
        } else {
            context.instruction(Instruction::Fizzle);
        }
        context.scope.intermediate(); // As is each subsequent parameter value
        context.label(next);
    }
    // The return value is a (backwards) list
    context.instruction(Instruction::Const(().into()));
    for _ in &rule.parameters {
        context
            .instruction(Instruction::Swap)
            .instruction(Instruction::Cons);
        context.scope.end_intermediate();
    }
    // Finally, put the return value into 'next()
    context
        .instruction(Instruction::Const(next.into()))
        .instruction(Instruction::Construct)
        .instruction(Instruction::Return);
    // This ends with 2 expected values on the stack ([state, retval])
    // so they are no longer intemediate
    context.scope.end_intermediate();

    // On failure, just return 'done
    context
        .label(on_done)
        .instruction(Instruction::Const(done.into()))
        .instruction(Instruction::Return);

    // Sad path: we have to undeclare all the variables that have been
    // declared so far, then fail as regular.
    for parameter in rule.parameters.iter().rev() {
        context.label(cleanup.pop().unwrap());
        context.undeclare_variables(parameter.bindings(), true);
    }
    context.jump(on_fail);

    context.label(end);
}
